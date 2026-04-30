#![no_std]
#![no_main]

mod api;
mod devices;
mod network;

use crate::api::{NUM_LISTENERS, NUM_NOTIFIERS};
use assign_resources::assign_resources;
use defmt::{Format, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use hoshiguma_api::{AccessControlRawInput, AccessControlState, BootReason};
use panic_probe as _;
use peek_o_display_bsp::{
    PeekODisplay,
    display::Rotation,
    embassy_rp::{
        gpio::{Level, Output},
        watchdog::Watchdog,
    },
    peripherals::{self, Peri},
    touch::Calibration,
};
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_19,
    },
    ethernet: EthernetResources {
        pio: PIO0,
        mosi: PIN_23,
        miso: PIN_22,
        sck: PIN_21,
        tx_dma: DMA_CH0,
        rx_dma: DMA_CH1,
        cs: PIN_20,
        int: PIN_24,
        reset: PIN_25,
    },
    access_control: AccessControlResources {
        granted: PIN_28,
        denied: PIN_27,
    },
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let board = PeekODisplay::default();
    let p = board.peripherals();
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    let net_stack = network::init(spawner, r.ethernet).await;

    let spi = board.board_spi();

    let display_rotation = Rotation::Deg0;
    let (display, backlight) = board.display(spi, display_rotation);
    let (touch, touch_irq) = board.touch(spi, display_rotation, Calibration::default());

    static NOTIFICATION_CHANNEL: Channel<CriticalSectionRawMutex, Notification, 8> = Channel::new();

    spawner.spawn(devices::display::task(display).unwrap());

    static BACKLIGHT_COMM: StaticCell<
        [devices::backlight::Channel; devices::backlight::NUM_COMM_CHANNELS],
    > = StaticCell::new();
    let backlight_comm = BACKLIGHT_COMM.init(Default::default());
    let backlight_comm_b = backlight_comm.each_ref().map(|comm| comm.side_b());
    spawner.spawn(devices::backlight::task(backlight, backlight_comm_b).unwrap());

    spawner.spawn(
        devices::touchscreen::task(touch, touch_irq, backlight_comm[NUM_LISTENERS].side_a())
            .unwrap(),
    );

    for idx in 0..NUM_LISTENERS {
        let comm = DeviceCommunicator {
            backlight: backlight_comm[idx].side_a(),
        };
        spawner.spawn(api::listen_task(net_stack, idx, comm).unwrap());
    }
    for idx in 0..NUM_NOTIFIERS {
        spawner.spawn(api::notify_task(net_stack, idx, NOTIFICATION_CHANNEL.receiver()).unwrap());
    }

    spawner.spawn(watchdog_feed_task(r.status).unwrap());
}

struct DeviceCommunicator {
    backlight: devices::backlight::TheirChannelSide,
}

#[derive(Format)]
enum Notification {
    PanelInteraction,
    AccessControlInputChanged(AccessControlRawInput),
    AccessControlStateChanged(AccessControlState),
}

impl Notification {
    fn expected_request_and_response(
        self,
    ) -> (
        hoshiguma_api::hmi::from_hmi::Request,
        hoshiguma_api::hmi::from_hmi::Response,
    ) {
        use hoshiguma_api::hmi::from_hmi::*;

        match self {
            Notification::PanelInteraction => (
                Request::NotifyPanelInteraction,
                Response(Ok(ResponseData::AckPanelInteraction)),
            ),
            Notification::AccessControlInputChanged(value) => (
                Request::NotifyAccessControlInputChanged(value.clone()),
                Response(Ok(ResponseData::AckAccessControlInputChanged(value))),
            ),
            Notification::AccessControlStateChanged(value) => (
                Request::NotifyAccessControlStateChanged(value.clone()),
                Response(Ok(ResponseData::AckAccessControlStateChanged(value))),
            ),
        }
    }
}

static COMM_GOOD_INDICATOR: Channel<CriticalSectionRawMutex, (), 8> = Channel::new();

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_secs(5));

    loop {
        let _ = COMM_GOOD_INDICATOR.receive().await;

        watchdog.feed(Duration::from_secs(5));

        // Blink the LED
        onboard_led.set_high();
        Timer::after_millis(10).await;
        onboard_led.set_low();
    }
}

fn boot_reason() -> BootReason {
    let reason = peek_o_display_bsp::embassy_rp::pac::WATCHDOG
        .reason()
        .read();

    if reason.force() {
        BootReason::WatchdogForced
    } else if reason.timer() {
        BootReason::WatchdogTimeout
    } else {
        BootReason::Normal
    }
}
