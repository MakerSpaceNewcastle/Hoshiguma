#![no_std]
#![no_main]

mod display;
mod telemetry;
mod ui_button;
mod wifi;

use defmt::unwrap;
use defmt_rtt as _;
use embassy_executor::{Executor, Spawner};
use embassy_rp::{
    gpio::{Level, Output},
    multicore::{spawn_core1, Stack},
    peripherals,
    watchdog::Watchdog,
};
use embassy_time::{Duration, Ticker, Timer};
use panic_probe as _;
use static_cell::StaticCell;

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

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
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

                unwrap!(spawner.spawn(crate::telemetry::task(r.telemetry_uart)));
            });
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        unwrap!(spawner.spawn(crate::ui_button::task(r.ui)));

        unwrap!(spawner.spawn(crate::display::state::task()));
        unwrap!(spawner.spawn(crate::display::task(r.display)));

        // TODO
        // unwrap!(spawner.spawn(crate::wifi::task(r.wifi, spawner)));
    });
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

    // Flash LED steadily to indicate normal operation
    let mut ticker = Ticker::every(steady_blink_delay);
    loop {
        watchdog.feed();
        led.toggle();
        ticker.next().await;
    }
}
