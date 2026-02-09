#![no_std]
#![no_main]

mod access_control;
mod api;
mod network;
mod ui;

use crate::api::{NUM_LISTENERS, NUM_NOTIFIERS};
use assign_resources::assign_resources;
use defmt::{Format, info};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Instant, Ticker, Timer};
use embedded_alloc::LlffHeap as Heap;
use hoshiguma_api::{
    BootReason,
    hmi::{AccessControlRawInput, AccessControlState},
};
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

#[global_allocator]
static HEAP: Heap = Heap::empty();

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
        granted: PIN_28, // Connect (via optoisolator) to output relay terminal on access controller
        denied: PIN_27, // Connect (via optoisolator) to denied LED terminal on access controller
    },
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    unsafe {
        embedded_alloc::init!(HEAP, 1024 * 96);
    }

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

    ui::init(spawner, display, touch, touch_irq, backlight);

    spawner.spawn(access_control::task(r.access_control).unwrap());

    for idx in 0..NUM_LISTENERS {
        spawner.spawn(api::listen_task(net_stack, idx).unwrap());
    }
    for idx in 0..NUM_NOTIFIERS {
        spawner.spawn(api::notify_task(net_stack, idx).unwrap());
    }

    spawner.spawn(watchdog_feed_task(r.status).unwrap());
}

#[derive(Format)]
enum Notification {
    PanelInteraction,
    AccessControlInputChanged(AccessControlRawInput),
    AccessControlStateChanged(AccessControlState),
}

static COMM_GOOD_INDICATOR: Channel<CriticalSectionRawMutex, (), 8> = Channel::new();

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_secs(5));

    let mut tick = Ticker::every(Duration::from_secs(1));

    loop {
        match select(tick.next(), COMM_GOOD_INDICATOR.receive()).await {
            Either::First(_) => {
                if Instant::now() < Instant::from_secs(60) {
                    watchdog.feed(Duration::from_secs(2));
                    continue;
                }
            }
            Either::Second(_) => {
                watchdog.feed(Duration::from_secs(5));

                // Blink the LED
                onboard_led.set_high();
                Timer::after_millis(10).await;
                onboard_led.set_low();
            }
        }
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
