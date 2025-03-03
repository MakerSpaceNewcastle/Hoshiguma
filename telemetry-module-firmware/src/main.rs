#![no_std]
#![no_main]

mod display;
mod telemetry;
mod ui_button;
mod wifi;

use core::sync::atomic::Ordering;

use crate::ui_button::{UiEvent, UI_INPUTS};
use defmt::{info, unwrap, warn};
use defmt_rtt as _;
use embassy_executor::raw::Executor;
use embassy_futures::select::{select, Either};
use embassy_rp::{
    gpio::{Level, Output},
    multicore::{spawn_core1, Stack},
    peripherals,
    watchdog::Watchdog,
};
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Instant, Ticker, Timer};
use panic_probe as _;
use portable_atomic::{self as _, AtomicU64};
use static_cell::StaticCell;

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

static mut STACK_CORE_1: Stack<4096> = Stack::new();

static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR_1: StaticCell<Executor> = StaticCell::new();

static SLEEP_TICKS_CORE_0: AtomicU64 = AtomicU64::new(0);
static SLEEP_TICKS_CORE_1: AtomicU64 = AtomicU64::new(0);

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(STACK_CORE_1) },
        move || {
            let executor_1 = EXECUTOR_1.init(Executor::new(usize::MAX as *mut ()));
            let spawner = executor_1.spawner();

            unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

            unwrap!(spawner.spawn(crate::telemetry::task(r.telemetry_uart)));

            unwrap!(spawner.spawn(crate::ui_button::task(r.ui)));

            unwrap!(spawner.spawn(report_cpu_usage()));

            loop {
                let before = Instant::now().as_ticks();
                cortex_m::asm::wfe();
                let after = Instant::now().as_ticks();
                SLEEP_TICKS_CORE_1.fetch_add(after - before, Ordering::Relaxed);
                unsafe { executor_1.poll() };
            }
        },
    );

    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    let spawner = executor_0.spawner();

    unwrap!(spawner.spawn(crate::display::state::task()));
    unwrap!(spawner.spawn(crate::display::task(r.display)));

    unwrap!(spawner.spawn(crate::wifi::task(r.wifi, spawner)));

    loop {
        let before = Instant::now().as_ticks();
        cortex_m::asm::wfe();
        let after = Instant::now().as_ticks();
        SLEEP_TICKS_CORE_0.fetch_add(after - before, Ordering::Relaxed);
        unsafe { executor_0.poll() };
    }
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

#[embassy_executor::task]
async fn report_cpu_usage() {
    let mut previous_tick = 0u64;
    let mut previous_sleep_tick_core_0 = 0u64;
    let mut previous_sleep_tick_core_1 = 0u64;

    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        ticker.next().await;

        let current_tick = Instant::now().as_ticks();
        let tick_difference = (current_tick - previous_tick) as f32;

        let current_sleep_tick_core_0 = SLEEP_TICKS_CORE_0.load(Ordering::Relaxed);
        let current_sleep_tick_core_1 = SLEEP_TICKS_CORE_1.load(Ordering::Relaxed);

        let calc_cpu_usage = |current_sleep_tick: u64, previous_sleep_tick: u64| -> f32 {
            let sleep_tick_difference = (current_sleep_tick - previous_sleep_tick) as f32;
            1f32 - sleep_tick_difference / tick_difference
        };

        let usage_core_0 = calc_cpu_usage(current_sleep_tick_core_0, previous_sleep_tick_core_0);
        let usage_core_1 = calc_cpu_usage(current_sleep_tick_core_1, previous_sleep_tick_core_1);

        previous_tick = current_tick;
        previous_sleep_tick_core_0 = current_sleep_tick_core_0;
        previous_sleep_tick_core_1 = current_sleep_tick_core_1;

        info!(
            "Usage: core 0 = {}, core 1 = {}",
            usage_core_0, usage_core_1
        );
    }
}
