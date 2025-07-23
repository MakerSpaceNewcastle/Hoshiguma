#![no_std]
#![no_main]

mod display;
mod metric;
mod network;
mod telemetry;
#[cfg(feature = "trace")]
mod trace;
mod ui_button;

use crate::ui_button::{UiEvent, UI_INPUTS};
use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::{
    gpio::{Level, Output},
    peripherals,
    watchdog::Watchdog,
    Peri,
};
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Instant, Timer};
use hoshiguma_protocol::types::{BootReason, SystemInformation};
use panic_probe as _;
use portable_atomic as _;

assign_resources::assign_resources! {
    display: DisplayResources {
        miso: PIN_12,
        mosi: PIN_11,
        clk: PIN_10,
        cs: PIN_6,
        dc: PIN_7,
        rst: PIN_8,
        led: PIN_9,
        spi: SPI1,
    }
    telemetry_uart: TelemetryUartResources {
        tx_pin: PIN_0,
        rx_pin: PIN_1,
        uart: UART0,
    }
    ui: UiResources {
        button: PIN_17,
    }
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_21,
    }
    wifi: WifiResources {
        pwr: PIN_23,
        cs: PIN_25,
        pio: PIO0,
        dio: PIN_24,
        clk: PIN_29,
        dma_ch: DMA_CH0,
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("{}", system_information());

    unwrap!(spawner.spawn(watchdog_feed_task(r.status)));
    unwrap!(spawner.spawn(crate::network::task(r.wifi, spawner)));
    unwrap!(spawner.spawn(crate::telemetry::system::task()));
    unwrap!(spawner.spawn(crate::telemetry::machine::task(r.telemetry_uart)));
    unwrap!(spawner.spawn(crate::ui_button::task(r.ui)));
    unwrap!(spawner.spawn(crate::display::task(r.display)));

    #[cfg(feature = "trace")]
    unwrap!(spawner.spawn(trace::task()));

    #[cfg(feature = "test-panic-on-core-0")]
    unwrap!(spawner.spawn(dummy_panic()));
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: crate::StatusResources) {
    #[cfg(feature = "trace")]
    trace::name_task("watchdog feed").await;

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_secs(5));

    let mut led = Output::new(r.led, Level::Low);

    let mut ui_event_rx = UI_INPUTS.subscriber().unwrap();

    loop {
        match select(Timer::after_secs(1), ui_event_rx.next_message()).await {
            Either::First(_) => {
                // Flash LED steadily to indicate normal operation
                watchdog.feed();
                led.toggle();
            }
            Either::Second(WaitResult::Lagged(_)) => unreachable!(),
            Either::Second(WaitResult::Message(UiEvent::ButtonPushedForALongTime)) => {
                info!("Triggering reboot!");
                watchdog.trigger_reset();
            }
            _ => {}
        }
    }
}

#[embassy_executor::task]
async fn dummy_panic() {
    embassy_time::Timer::after_secs(5).await;
    panic!("oh dear, how sad. nevermind...");
}

fn system_information() -> SystemInformation {
    SystemInformation {
        git_revision: git_version::git_version!().try_into().unwrap(),
        last_boot_reason: boot_reason(),
        uptime_milliseconds: Instant::now().as_millis(),
    }
}

fn boot_reason() -> BootReason {
    let reason = embassy_rp::pac::WATCHDOG.reason().read();

    if reason.force() {
        BootReason::WatchdogForced
    } else if reason.timer() {
        BootReason::WatchdogTimeout
    } else {
        BootReason::Normal
    }
}
