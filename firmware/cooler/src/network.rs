use crate::{
    EthernetResources, MachineControl,
    devices::{
        compressor::CompressorInterfaceChannel, coolant_pump::CoolantPumpInterfaceChannel,
        coolant_rate_sensors::CoolantRateInterfaceChannel,
        radiator_fan::RadiatorFanInterfaceChannel,
        temperature_sensors::TemperatureInterfaceChannel,
    },
};
use core::net::Ipv4Addr;
use defmt::{debug, info, warn};
use embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice;
use embassy_executor::Spawner;
use embassy_net::{ConfigV4, Ipv4Cidr, Stack, StackResources, StaticConfigV4, tcp::TcpSocket};
use embassy_net_wiznet::{Device, Runner, State, chip::W5500};
use embassy_rp::{
    bind_interrupts,
    clocks::RoscRng,
    gpio::{Input, Level, Output, Pull},
    peripherals::{DMA_CH0, DMA_CH1, PIO0},
    pio::Pio,
    pio_programs::spi::Spi,
    spi::Config as SpiConfig,
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use embassy_time::{Duration, Instant};
use embedded_io_async::Write;
use heapless::Vec;
use hoshiguma_api::cooler::{Request, Response, ResponseData};
use static_cell::StaticCell;

const COOLER_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 4);

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

pub(crate) const NUM_LISTENERS: usize = 3;

#[embassy_executor::task]
pub(super) async fn task(
    spawner: Spawner,
    r: EthernetResources,
    mut machine: heapless::Vec<MachineControl, NUM_LISTENERS>,
) -> ! {
    let mut rng = RoscRng;

    let mut spi_cfg = SpiConfig::default();
    spi_cfg.frequency = 12_500_000;

    let Pio {
        mut common, sm0, ..
    } = Pio::new(r.pio, Irqs);

    let spi = Spi::new(
        &mut common,
        sm0,
        r.sck,
        r.mosi,
        r.miso,
        r.tx_dma,
        r.rx_dma,
        Irqs,
        spi_cfg,
    );
    let cs = Output::new(r.cs, Level::High);
    let w5500_int = Input::new(r.int, Pull::Up);
    let w5500_reset = Output::new(r.reset, Level::High);

    static SPI: StaticCell<
        Mutex<CriticalSectionRawMutex, Spi<'static, PIO0, 0, embassy_rp::spi::Async>>,
    > = StaticCell::new();
    let spi = SPI.init(Mutex::new(spi));

    let mac_addr = [0x02, 0x00, 0x00, 0xff, 0x22, 0x22];
    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());
    let device = SpiDevice::new(spi, cs);
    let (device, runner) = embassy_net_wiznet::new(mac_addr, state, device, w5500_int, w5500_reset)
        .await
        .unwrap();
    spawner.spawn(ethernet_task(runner).unwrap());

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<12>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        device,
        embassy_net::Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    stack.set_config_v4(ConfigV4::Static(StaticConfigV4 {
        address: Ipv4Cidr::new(COOLER_IP_ADDRESS, 24),
        gateway: Some(Ipv4Addr::new(10, 69, 69, 1)),
        dns_servers: Vec::new(),
    }));

    spawner.spawn(net_task(runner).unwrap());

    for i in 0..NUM_LISTENERS {
        spawner.spawn(listen_task(stack, i as u8, 1234, machine.pop().unwrap()).unwrap());
    }

    loop {
        embassy_time::Timer::after_secs(10).await;
    }
}

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<
        'static,
        W5500,
        SpiDevice<
            'static,
            CriticalSectionRawMutex,
            Spi<'static, PIO0, 0, embassy_rp::spi::Async>,
            Output<'static>,
        >,
        Input<'static>,
        Output<'static>,
    >,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
async fn listen_task(stack: Stack<'static>, id: u8, port: u16, mut machine: MachineControl) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut buf = [0; 4096];

    loop {
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(5)));

        info!("socket {}: Listening on TCP:{}...", id, port);
        if let Err(e) = socket.accept(port).await {
            warn!("socket {}: accept error: {:?}", id, e);
            continue;
        }
        info!(
            "socket {}: connection from {:?}",
            id,
            socket.remote_endpoint()
        );

        loop {
            let n = match socket.read(&mut buf).await {
                Ok(0) => {
                    info!("socket {}: EOF", id);
                    break;
                }
                Ok(n) => n,
                Err(e) => {
                    warn!("socket {}: {:?}", id, e);
                    break;
                }
            };

            let received = &mut buf[..n];
            debug!("socket {}: received {} bytes", id, received.len());

            let request = match postcard::from_bytes_cobs::<Request>(received) {
                Ok(request) => request,
                Err(_) => {
                    warn!("socket {}: failed to parse request", id);
                    continue;
                }
            };

            let _ = crate::COMM_GOOD_INDICATOR.try_send(());

            let response = match request {
                Request::GetGitRevision => Response(Ok(ResponseData::GitRevision(
                    git_version::git_version!().try_into().unwrap(),
                ))),
                Request::GetUptime => Response(Ok(ResponseData::Uptime(
                    Instant::now().duration_since(Instant::MIN).into(),
                ))),
                Request::GetBootReason => {
                    Response(Ok(ResponseData::BootReason(crate::boot_reason())))
                }
                Request::GetRadiatorFanState => Response(
                    machine
                        .radiator_fan
                        .get()
                        .await
                        .map(|state| ResponseData::RadiatorFanState(state))
                        .map_err(|_| ()),
                ),
                Request::SetRadiatorFanState(state) => Response(
                    machine
                        .radiator_fan
                        .set(state)
                        .await
                        .map(|state| ResponseData::RadiatorFanState(state))
                        .map_err(|_| ()),
                ),
                Request::GetCompressorState => Response(
                    machine
                        .compressor
                        .get()
                        .await
                        .map(|state| ResponseData::CompressorState(state))
                        .map_err(|_| ()),
                ),
                Request::SetCompressorState(state) => Response(
                    machine
                        .compressor
                        .set(state)
                        .await
                        .map(|state| ResponseData::CompressorState(state))
                        .map_err(|_| ()),
                ),
                Request::GetCoolantPumpState => Response(
                    machine
                        .coolant_pump
                        .get()
                        .await
                        .map(|state| ResponseData::CoolantPumpState(state))
                        .map_err(|_| ()),
                ),
                Request::SetCoolantPumpState(state) => Response(
                    machine
                        .coolant_pump
                        .set(state)
                        .await
                        .map(|state| ResponseData::CoolantPumpState(state))
                        .map_err(|_| ()),
                ),
                Request::GetTemperatures => Response(
                    machine
                        .temperatures
                        .get()
                        .await
                        .map(|v| ResponseData::Temperatures(v))
                        .map_err(|_| ()),
                ),
                Request::GetCoolantFlowRate => Response(
                    machine
                        .coolant_flow_rate
                        .get()
                        .await
                        .map(|v| ResponseData::CoolantFlowRate(v))
                        .map_err(|_| ()),
                ),
                Request::GetCoolantReturnRate => Response(
                    machine
                        .coolant_return_rate
                        .get()
                        .await
                        .map(|v| ResponseData::CoolantReturnRate(v))
                        .map_err(|_| ()),
                ),
            };

            let response_bytes = match postcard::to_slice_cobs(&response, &mut buf) {
                Ok(bytes) => bytes,
                Err(_) => {
                    warn!("socket {}: failed to serialize response", id);
                    continue;
                }
            };

            if let Err(e) = socket.write_all(&response_bytes).await {
                warn!("socket {}: write error: {:?}", id, e);
                break;
            }
        }
    }
}
