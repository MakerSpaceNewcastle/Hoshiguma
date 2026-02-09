use embassy_embedded_hal::shared_bus::asynch::spi::SpiDeviceWithConfig;
use embassy_executor::Spawner;
use embassy_net::{Config, Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embassy_net_wiznet::{Device, Runner, State, chip::W5500};
use embassy_rp::{
    bind_interrupts,
    clocks::RoscRng,
    dma,
    gpio::{Input, Level, Output, Pull},
    peripherals::{DMA_CH0, DMA_CH1, DMA_CH2, DMA_CH3, SPI0, SPI1},
    spi::{self, Spi},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, mutex::Mutex};
use heapless::Vec;
use hoshiguma_api::TELEMETRY_BRIDGE_IP_ADDRESS;
use hoshiguma_common::network::{
    TELEMETRY_BRIDGE_MAC_ADDRESS, TELEMETRY_BRIDGE_MAC_ADDRESS_PUBLIC,
};
use static_cell::StaticCell;

fn w5500_spi_config() -> spi::Config {
    let mut spi_config = spi::Config::default();
    spi_config.frequency = 50_000_000;
    spi_config.phase = spi::Phase::CaptureOnSecondTransition;
    spi_config.polarity = spi::Polarity::IdleHigh;
    spi_config
}

bind_interrupts!(struct Irqs {
    DMA_IRQ_0 => dma::InterruptHandler<DMA_CH0>, dma::InterruptHandler<DMA_CH1>, dma::InterruptHandler<DMA_CH2>, dma::InterruptHandler<DMA_CH3>;
});

pub(crate) async fn init_internal(
    r: crate::Ethernet1Resources,
    spawner: Spawner,
) -> Stack<'static> {
    let spi = Spi::new(
        r.spi,
        r.clk,
        r.mosi,
        r.miso,
        r.tx_dma,
        r.rx_dma,
        Irqs,
        w5500_spi_config(),
    );

    static SPI: StaticCell<
        Mutex<CriticalSectionRawMutex, Spi<'static, SPI0, embassy_rp::spi::Async>>,
    > = StaticCell::new();
    let spi = SPI.init(Mutex::new(spi));

    let cs = Output::new(r.cs_pin, Level::High);
    let device = SpiDeviceWithConfig::new(spi, cs, w5500_spi_config());

    let w5500_int = Input::new(r.int_pin, Pull::Up);
    let w5500_reset = Output::new(r.rst_pin, Level::High);

    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());

    let (device, runner) = embassy_net_wiznet::new(
        TELEMETRY_BRIDGE_MAC_ADDRESS,
        state,
        device,
        w5500_int,
        w5500_reset,
    )
    .await
    .unwrap();
    spawner.spawn(ethernet_1_task(runner).unwrap());

    static RESOURCES: StaticCell<StackResources<8>> = StaticCell::new();
    let mut rng = RoscRng;
    let (stack, runner) = embassy_net::new(
        device,
        Config::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::new(TELEMETRY_BRIDGE_IP_ADDRESS, 24),
            gateway: None,
            dns_servers: Vec::default(),
        }),
        RESOURCES.init(StackResources::new()),
        rng.next_u64(),
    );
    spawner.spawn(net_task(runner).unwrap());

    stack
}

pub(crate) async fn init_external(
    r: crate::Ethernet2Resources,
    spawner: Spawner,
) -> Stack<'static> {
    let spi = Spi::new(
        r.spi,
        r.clk,
        r.mosi,
        r.miso,
        r.tx_dma,
        r.rx_dma,
        Irqs,
        w5500_spi_config(),
    );

    static SPI: StaticCell<
        Mutex<CriticalSectionRawMutex, Spi<'static, SPI1, embassy_rp::spi::Async>>,
    > = StaticCell::new();
    let spi = SPI.init(Mutex::new(spi));

    let cs = Output::new(r.cs_pin, Level::High);
    let device = SpiDeviceWithConfig::new(spi, cs, w5500_spi_config());

    let w5500_int = Input::new(r.int_pin, Pull::Up);
    let w5500_reset = Output::new(r.rst_pin, Level::High);

    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());

    let (device, runner) = embassy_net_wiznet::new(
        TELEMETRY_BRIDGE_MAC_ADDRESS_PUBLIC,
        state,
        device,
        w5500_int,
        w5500_reset,
    )
    .await
    .unwrap();
    spawner.spawn(ethernet_2_task(runner).unwrap());

    static RESOURCES: StaticCell<StackResources<8>> = StaticCell::new();
    let mut rng = RoscRng;
    let (stack, runner) = embassy_net::new(
        device,
        Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::new()),
        rng.next_u64(),
    );
    spawner.spawn(net_task(runner).unwrap());

    stack
}

type EthernetSpi<SPI> = SpiDeviceWithConfig<
    'static,
    CriticalSectionRawMutex,
    Spi<'static, SPI, embassy_rp::spi::Async>,
    Output<'static>,
>;

#[embassy_executor::task]
async fn ethernet_1_task(
    runner: Runner<'static, W5500, EthernetSpi<SPI0>, Input<'static>, Output<'static>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn ethernet_2_task(
    runner: Runner<'static, W5500, EthernetSpi<SPI1>, Input<'static>, Output<'static>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task(pool_size = 2)]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}
