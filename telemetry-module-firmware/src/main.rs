#![no_std]
#![no_main]

mod display;
mod network;
mod telemetry;
mod ui_button;

use crate::ui_button::{UiEvent, UI_INPUTS};
use defmt::{info, unwrap, warn};
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_rp::{
    gpio::{Level, Output},
    peripherals,
    watchdog::Watchdog,
};
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Ticker, Timer};
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
        rx_pin: PIN_1,
        uart: UART0,
        dma_ch: DMA_CH1,
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

    unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

    unwrap!(spawner.spawn(crate::telemetry::task(r.telemetry_uart)));

    unwrap!(spawner.spawn(crate::ui_button::task(r.ui)));

    unwrap!(spawner.spawn(crate::display::state::task()));
    unwrap!(spawner.spawn(crate::display::task(r.display)));

    unwrap!(spawner.spawn(crate::network::task(r.wifi, spawner)));
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: crate::StatusResources) {
    let steady_blink_delay = Duration::from_hz(1);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(steady_blink_delay + Duration::from_millis(100));

    let mut led = Output::new(r.led, Level::Low);

    // Flash LED fast to indicate boot
    for _ in 0..10 {
        watchdog.feed();
        led.toggle();
        Timer::after_millis(50).await;
    }

    let mut ui_event_rx = UI_INPUTS.subscriber().unwrap();
    let mut ticker = Ticker::every(steady_blink_delay);

    loop {
        match select(ticker.next(), ui_event_rx.next_message()).await {
            Either::First(_) => {
                // Flash LED steadily to indicate normal operation
                watchdog.feed();
                led.toggle();
            }
            Either::Second(WaitResult::Lagged(lost_messages)) => {
                warn!("Subscriber lagged, lost {} messages", lost_messages);
            }
            Either::Second(WaitResult::Message(UiEvent::ButtonPushedForALongTime)) => {
                info!("Triggering reboot!");
                watchdog.trigger_reset();
            }
            _ => {}
        }
    }
}
