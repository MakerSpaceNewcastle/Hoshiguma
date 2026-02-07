pub(crate) mod telemetry_tx;
pub(crate) mod time;

use crate::{
    TELEMETRY_TX,
    self_telemetry::{DATA_POINTS_ACCEPTED, DATA_POINTS_DISCARDED},
};
use core::{cell::RefCell, sync::atomic::Ordering};
use defmt::{info, warn};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_net::{
    Config, Stack, StackResources, StaticConfigV4,
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
};
use embassy_net_wiznet::{Device, Runner, State, chip::W5500};
use embassy_rp::{
    clocks::RoscRng,
    gpio::{Input, Level, Output, Pull},
    peripherals::SPI0,
    spi::Spi,
};
use embassy_sync::{
    blocking_mutex::{CriticalSectionMutex, raw::CriticalSectionRawMutex},
    mutex::Mutex,
    pubsub::WaitResult,
};
use embassy_time::{Duration, Instant, Timer};
use reqwless::client::{HttpClient, TlsConfig, TlsVerify};
use static_cell::StaticCell;
use telemetry_tx::TelegrafBuffer;

pub(crate) static LINK_STATE: CriticalSectionMutex<RefCell<LinkState>> =
    CriticalSectionMutex::new(RefCell::new(LinkState {
        last_changed: None,
        dhcp4_config: None,
    }));

#[derive(Clone)]
pub(crate) struct LinkState {
    last_changed: Option<Instant>,
    pub(crate) dhcp4_config: Option<StaticConfigV4>,
}

impl LinkState {
    pub(crate) fn age(&self) -> Duration {
        Instant::now() - self.last_changed.unwrap_or(Instant::MIN)
    }
}

pub(super) async fn init(r: crate::EthernetResources, spawner: Spawner) -> Stack<'static> {
    let mut spi_config = embassy_rp::spi::Config::default();
    spi_config.frequency = 50_000_000;
    spi_config.phase = embassy_rp::spi::Phase::CaptureOnSecondTransition;
    spi_config.polarity = embassy_rp::spi::Polarity::IdleHigh;

    let spi = Spi::new(
        r.spi,
        r.clk,
        r.mosi,
        r.miso,
        r.tx_dma,
        r.rx_dma,
        spi_config.clone(),
    );

    static SPI: StaticCell<
        Mutex<CriticalSectionRawMutex, Spi<'static, SPI0, embassy_rp::spi::Async>>,
    > = StaticCell::new();
    let spi = SPI.init(Mutex::new(spi));

    let cs = Output::new(r.cs_pin, Level::High);
    let device = SpiDeviceWithConfig::new(spi, cs, spi_config);

    let w5500_int = Input::new(r.int_pin, Pull::Up);
    let w5500_reset = Output::new(r.rst_pin, Level::High);

    let mac_addr = [0x02, 0x00, 0x00, 0x00, 0x00, 0x00];

    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());

    let (device, runner) = embassy_net_wiznet::new(mac_addr, state, device, w5500_int, w5500_reset)
        .await
        .unwrap();

    spawner.must_spawn(ethernet_task(runner));

    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
    let mut rng = RoscRng;
    let (stack, runner) = embassy_net::new(
        device,
        Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::<4>::new()),
        rng.next_u64(),
    );
    spawner.must_spawn(net_task(runner));

    spawner.must_spawn(connection_task(stack));

    stack
}

type EthernetSpi = SpiDeviceWithConfig<
    'static,
    CriticalSectionRawMutex,
    Spi<'static, SPI0, embassy_rp::spi::Async>,
    Output<'static>,
>;

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<'static, W5500, EthernetSpi, Input<'static>, Output<'static>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn connection_task(stack: Stack<'static>) -> ! {
    let mut rng = RoscRng;

    'connection: loop {
        // Get configuration via DHCP
        {
            info!("Waiting for DHCP");
            while !stack.is_config_up() {
                Timer::after_millis(100).await;
            }
            info!("DHCP is now up");

            let config = stack.config_v4().unwrap();
            LINK_STATE.lock(|v| {
                let mut state = v.borrow_mut();
                state.last_changed.replace(Instant::now());
                state.dhcp4_config.replace(config);
            });
        }

        let mut rx_buffer = [0; 8192];
        let mut tls_read_buffer = [0; 16640];
        let mut tls_write_buffer = [0; 16640];

        let client_state = TcpClientState::<1, 1024, 1024>::new();
        let tcp_client = TcpClient::new(stack, &client_state);
        let dns_client = DnsSocket::new(stack);
        let tls_config = TlsConfig::new(
            rng.next_u64(),
            &mut tls_read_buffer,
            &mut tls_write_buffer,
            TlsVerify::None,
        );

        let mut http_client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls_config);

        let mut data_point_line_rx = TELEMETRY_TX.subscriber().unwrap();

        let mut telegraf_buffer = TelegrafBuffer::default();

        loop {
            match select(data_point_line_rx.next_message(), Timer::after_millis(800)).await {
                Either::First(WaitResult::Message(metric)) => {
                    // Add the metric to the buffer
                    match telegraf_buffer.push(metric) {
                        Ok(_) => {
                            DATA_POINTS_ACCEPTED.add(1, Ordering::Relaxed);
                        }
                        Err(_) => {
                            warn!("Failed to push metric to buffer");
                            DATA_POINTS_DISCARDED.add(1, Ordering::Relaxed);
                        }
                    }

                    // If the buffer is nearing capacity, then send now
                    if telegraf_buffer.send_required() {
                        info!("Tx reason: buffer nearly full");
                        telegraf_buffer.tx(&mut http_client, &mut rx_buffer).await;
                    }
                }
                Either::First(WaitResult::Lagged(_)) => unreachable!(),
                Either::Second(_) => {
                    info!("Tx reason: periodic purge");
                    telegraf_buffer.tx(&mut http_client, &mut rx_buffer).await;

                    if !stack.is_config_up() {
                        warn!("Network down");

                        LINK_STATE.lock(|v| {
                            let mut state = v.borrow_mut();
                            state.last_changed.replace(Instant::now());
                            state.dhcp4_config.take();
                        });

                        continue 'connection;
                    }
                }
            }
        }
    }
}
