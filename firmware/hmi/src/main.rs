#![no_std]
#![no_main]

mod network;

use assign_resources::assign_resources;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Ticker, Timer};
use embedded_graphics::{draw_target::DrawTarget, pixelcolor::Rgb666, prelude::RgbColor};
use hoshiguma_api::BootReason;
use panic_probe as _;
use peek_o_display_bsp::{
    PeekODisplay,
    display::Rotation,
    embassy_rp::{
        gpio::{Input, Level, Output, Pull},
        watchdog::Watchdog,
    },
    peripherals::{self, Peri},
    touch::Calibration,
};

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

    let spi = board.board_spi();

    // TODO
    let display_rotation = Rotation::Deg0;
    let (mut display, _backlight) = board.display(spi, display_rotation);
    let (mut touch, touch_irq) = board.touch(spi, display_rotation, Calibration::default());
    display.clear(Rgb666::BLACK).unwrap();
    touch.read();
    let access_control_signal = Input::new(r.access_control.granted, Pull::None);

    let mut comm = heapless::Vec::new();
    for i in 0..network::NUM_LISTENERS {
        if comm
            .push(DeviceCommunicator {
                // TODO
            })
            .is_err()
        {
            panic!();
        }
    }
    network::init(spawner, r.ethernet, comm).await;

    spawner.spawn(watchdog_feed_task(r.status).unwrap());
}

struct DeviceCommunicator;

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
