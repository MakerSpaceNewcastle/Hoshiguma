use crate::{DeviceCommunicator, EthernetResources};
use defmt::warn;
use embassy_executor::Spawner;
use embassy_net::{Ipv4Cidr, Stack, StackResources, StaticConfigV4};
use embassy_net_wiznet::{Device, Runner, State, chip::W5500};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver, mutex::Mutex};
use embassy_time::Instant;
use heapless::Vec;
use hoshiguma_api::{
    Message,
    hmi::{Notification, Request, Response, ResponseData},
};
use hoshiguma_common::network::{
    NotificationSubscriptionChannel, NotificationSubscriptionChannelPublisher,
    NotificationSubscriptionChannelSubscriber, Subscription,
    config::{HMI_IP_ADDRESS, HMI_MAC_ADDRESS},
    message_handler_loop, notification_tx_loop,
};
use peek_o_display_bsp::embassy_rp::{
    self, bind_interrupts,
    clocks::RoscRng,
    gpio::{Input, Level, Output, Pull},
    peripherals::{DMA_CH0, DMA_CH1, PIO0},
    pio::Pio,
    pio_programs::spi::Spi,
    spi::Config as SpiConfig,
};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => embassy_rp::pio::InterruptHandler<PIO0>;
    DMA_IRQ_0 => embassy_rp::dma::InterruptHandler<DMA_CH0>, embassy_rp::dma::InterruptHandler<DMA_CH1>;
});

pub(crate) const NUM_LISTENERS: usize = 3;
pub(crate) const NUM_NOTIFIERS: usize = 2;

type SpiDevice = embassy_embedded_hal::shared_bus::asynch::spi::SpiDevice<
    'static,
    CriticalSectionRawMutex,
    Spi<'static, PIO0, 0, embassy_rp::spi::Async>,
    Output<'static>,
>;

pub(super) async fn init(
    spawner: Spawner,
    r: EthernetResources,
    notif_rx: Receiver<'static, CriticalSectionRawMutex, Notification, 8>,
    mut comm: Vec<DeviceCommunicator, NUM_LISTENERS>,
) {
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
        embassy_net_wiznet::new(HMI_MAC_ADDRESS, state, device, w5500_int, w5500_reset)
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
            address: Ipv4Cidr::new(HMI_IP_ADDRESS, 24),
            gateway: None,
            dns_servers: Vec::new(),
        }),
        RESOURCES.init(StackResources::new()),
        seed,
    );

    spawner.spawn(net_task(runner).unwrap());

    static NOTIF_SUB_COMM: NotificationSubscriptionChannel<NUM_NOTIFIERS> =
        NotificationSubscriptionChannel::new();

    for i in 0..NUM_LISTENERS {
        spawner.spawn(
            listen_task(
                stack,
                i as u8,
                NOTIF_SUB_COMM.publisher().unwrap(),
                comm.pop().unwrap(),
            )
            .unwrap(),
        );
    }
    for i in 0..NUM_NOTIFIERS {
        spawner.spawn(
            notify_task(
                stack,
                i as u8,
                NOTIF_SUB_COMM.subscriber().unwrap(),
                notif_rx.clone(),
            )
            .unwrap(),
        );
    }
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

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
async fn listen_task(
    stack: Stack<'static>,
    id: u8,
    notif_sub_tx: NotificationSubscriptionChannelPublisher<NUM_NOTIFIERS>,
    mut comm: DeviceCommunicator,
) {
    message_handler_loop(stack, id, async |mut message| {
        let request = match message.payload::<Request>() {
            Ok(request) => request,
            Err(_) => {
                warn!("socket {}: failed to parse request", id);
                return Message::new(&Response(Err(()))).unwrap();
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
            Request::GetBootReason => Response(Ok(ResponseData::BootReason(crate::boot_reason()))),
            Request::SubscribeToNotifications(ip) => {
                let subscription = Subscription { ip };
                notif_sub_tx.publish(subscription).await;
                Response(Ok(ResponseData::SubscribedToNotifications))
            }
        };

        match Message::new(&response) {
            Ok(message) => message,
            Err(_) => {
                warn!("socket {}: failed to serialize response", id);
                Message::new(&Response(Err(()))).unwrap()
            }
        }
    })
    .await
}

#[embassy_executor::task(pool_size = NUM_NOTIFIERS)]
async fn notify_task(
    stack: Stack<'static>,
    id: u8,
    sub_rx: NotificationSubscriptionChannelSubscriber<NUM_NOTIFIERS>,
    notif_rx: Receiver<'static, CriticalSectionRawMutex, Notification, 8>,
) {
    notification_tx_loop(stack, id, sub_rx, notif_rx).await
}
