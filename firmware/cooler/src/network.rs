use crate::EthernetResources;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Stack, StackResources, StaticConfigV4};
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
use heapless::Vec;
use hoshiguma_api::COOLER_IP_ADDRESS;
use hoshiguma_common::network::COOLER_MAC_ADDRESS;
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

type SpiDevice = embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice<
    'static,
    CriticalSectionRawMutex,
    Spi<'static, PIO0, 0, embassy_rp::spi::Async>,
    Output<'static>,
>;

pub(super) async fn init(spawner: Spawner, r: EthernetResources) -> Stack<'static> {
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

    static STATE: StaticCell<State<8, 8>> = StaticCell::new();
    let state = STATE.init(State::<8, 8>::new());
    let device = SpiDevice::new(spi, cs);
    let (device, runner) =
        embassy_net_wiznet::new(COOLER_MAC_ADDRESS, state, device, w5500_int, w5500_reset)
            .await
            .unwrap();
    spawner.spawn(ethernet_task(runner).unwrap());

    // Generate random seed
    let seed = rng.next_u64();

    // Init network stack
    static RESOURCES: StaticCell<StackResources<12>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        device,
        embassy_net::Config::ipv4_static(StaticConfigV4 {
            address: Ipv4Cidr::new(COOLER_IP_ADDRESS, 24),
            gateway: None,
            dns_servers: Vec::new(),
        }),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    spawner.spawn(net_task(runner).unwrap());

    stack
}

#[embassy_executor::task]
async fn ethernet_task(
    runner: Runner<'static, W5500, SpiDevice, Input<'static>, Output<'static>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, Device<'static>>) -> ! {
    runner.run().await
}
