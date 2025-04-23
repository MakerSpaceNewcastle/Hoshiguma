pub(crate) mod telemetry_tx;
pub(crate) mod time;

use core::{cell::RefCell, sync::atomic::Ordering};
use cyw43::{JoinOptions, PowerManagementMode, State};
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::{
    dns::DnsSocket,
    tcp::client::{TcpClient, TcpClientState},
    Config, StackResources, StaticConfigV4,
};
use embassy_rp::{
    bind_interrupts,
    clocks::RoscRng,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIO0},
    pio::{InterruptHandler, Pio},
};
use embassy_sync::{blocking_mutex::CriticalSectionMutex, pubsub::WaitResult};
use embassy_time::Timer;
use rand::RngCore;
use reqwless::client::{HttpClient, TlsConfig, TlsVerify};
use static_cell::StaticCell;
use telemetry_tx::{
    MetricBuffer, METRIC_TX, TELEMETRY_TX_BUFFER_SUBMISSIONS, TELEMETRY_TX_FAIL_BUFFER,
};

pub(crate) const WIFI_SSID: &str = env!("WIFI_SSID");

pub(crate) static DHCP_CONFIG: CriticalSectionMutex<RefCell<Option<StaticConfigV4>>> =
    CriticalSectionMutex::new(RefCell::new(None));

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("cyw43").await;

    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("net stack").await;

    runner.run().await
}

#[embassy_executor::task]
pub(super) async fn task(r: crate::WifiResources, spawner: Spawner) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("net init").await;

    let pwr = Output::new(r.pwr, Level::Low);
    let cs = Output::new(r.cs, Level::High);

    let mut pio = Pio::new(r.pio, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        r.dio,
        r.clk,
        r.dma_ch,
    );

    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());

    let fw = include_bytes!("../../cyw43-firmware/43439A0.bin");
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    let clm = include_bytes!("../../cyw43-firmware/43439A0_clm.bin");
    control.init(clm).await;

    control
        .set_power_management(PowerManagementMode::PowerSave)
        .await;

    let mut rng = RoscRng;
    let seed = rng.next_u64();

    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::<4>::new()),
        seed,
    );
    unwrap!(spawner.spawn(net_task(runner)));

    info!("Joining WiFi network {}", WIFI_SSID);
    loop {
        match control
            .join(
                WIFI_SSID,
                JoinOptions::new(env!("WIFI_PASSWORD").as_bytes()),
            )
            .await
        {
            Ok(_) => break,
            Err(err) => {
                warn!("Failed to join WiFi network with status {}", err.status);
            }
        }
    }

    // Get configuration via DHCP
    {
        info!("Waiting for DHCP");
        while !stack.is_config_up() {
            Timer::after_millis(100).await;
        }
        info!("DHCP is now up");

        let config = stack.config_v4().unwrap();
        DHCP_CONFIG.lock(|v| {
            v.borrow_mut().replace(config);
        });
    }

    let mut rng = RoscRng;
    let seed = rng.next_u64();

    let mut rx_buffer = [0; 8192];
    let mut tls_read_buffer = [0; 16640];
    let mut tls_write_buffer = [0; 16640];

    let client_state = TcpClientState::<1, 1024, 1024>::new();
    let tcp_client = TcpClient::new(stack, &client_state);
    let dns_client = DnsSocket::new(stack);
    let tls_config = TlsConfig::new(
        seed,
        &mut tls_read_buffer,
        &mut tls_write_buffer,
        TlsVerify::None,
    );

    let mut http_client = HttpClient::new_with_tls(&tcp_client, &dns_client, tls_config);

    let mut metric_rx = METRIC_TX.subscriber().unwrap();

    let mut metric_buffer = MetricBuffer::default();

    // Initial time sync
    let mut attempt = 0;
    'initial_time_sync: loop {
        attempt += 1;
        info!("Initial time sync, attempt {}", attempt);

        time::time_sync(stack).await;

        if time::wall_time().is_some() {
            break 'initial_time_sync;
        } else {
            Timer::after_secs(1).await;
        }
    }

    loop {
        match select(metric_rx.next_message(), Timer::after_secs(2)).await {
            Either::First(WaitResult::Message(metric)) => {
                // Add the metric to the buffer
                match metric_buffer.push(metric) {
                    Ok(_) => {
                        TELEMETRY_TX_BUFFER_SUBMISSIONS.add(1, Ordering::Relaxed);
                    }
                    Err(_) => {
                        warn!("Failed to push metric to buffer");
                        TELEMETRY_TX_FAIL_BUFFER.add(1, Ordering::Relaxed);
                    }
                }

                // If the buffer is nearing capacity, then send now
                if metric_buffer.send_required() {
                    info!("Tx reason: buffer nearly full");
                    metric_buffer.tx(&mut http_client, &mut rx_buffer).await;
                }
            }
            Either::First(WaitResult::Lagged(_)) => unreachable!(),
            Either::Second(_) => {
                info!("Tx reason: periodic purge");
                metric_buffer.tx(&mut http_client, &mut rx_buffer).await;
            }
        }
    }
}
