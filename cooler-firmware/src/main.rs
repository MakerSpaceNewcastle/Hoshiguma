#![no_std]
#![no_main]

use assign_resources::assign_resources;
use core::sync::atomic::Ordering;
use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::raw::Executor;
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    multicore::{spawn_core1, Stack},
    watchdog::Watchdog,
};
use embassy_time::{Duration, Instant, Ticker, Timer};
use git_version::git_version;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::peripherals::{self, PicoPlc};
use portable_atomic::{AtomicBool, AtomicU64};
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    onewire: OnewireResources {
        pin: ONEWIRE,
    },
    mains_detect: MainsDetectResources {
        detect: IN_1,
    },
    flow_sensor: FlowSensorResources {
        pwm: PWM_SLICE7,
        pin: IN_0,
    },
    resevoir_level: ResevoirLevelSensorResources {
        low: IN_2,
    },
    header_tank_level: HeaderTankLevelSensorResources {
        empty: IN_3,
        low : IN_4,
    },
    relays: RelayOutputResources {
        fan: RELAY_0,
        compressor: RELAY_1,
        stirrer: RELAY_2,
        pump: RELAY_3,
    },
    telemetry: ControlCommunicationResources {
        tx_pin: IO_0,
        rx_pin: IO_1,
        uart: UART0,
        dma_ch: DMA_CH0,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    // Flag the panic, indicating that executors should stop scheduling work
    PANIC_HALT.store(true, Ordering::Relaxed);

    let p = unsafe { PicoPlc::steal() };
    let r = split_resources!(p);

    // TODO

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed();

        // Blink the on-board LED pretty fast
        led.toggle();

        embassy_time::block_for(Duration::from_millis(50));
    }
}

static mut CORE_1_STACK: Stack<4096> = Stack::new();

static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR_1: StaticCell<Executor> = StaticCell::new();

static SLEEP_TICKS_CORE_0: AtomicU64 = AtomicU64::new(0);
static SLEEP_TICKS_CORE_1: AtomicU64 = AtomicU64::new(0);

static PANIC_HALT: AtomicBool = AtomicBool::new(false);

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = PicoPlc::default();
    let r = split_resources!(p);

    info!("Version: {}", git_version!());

    // Unused IO
    let _in5 = Input::new(p.IN_5, Pull::Down);
    let _in6 = Input::new(p.IN_6, Pull::Down);
    let _in7 = Input::new(p.IN_7, Pull::Down);
    let _relay4 = Output::new(p.RELAY_4, Level::Low);
    let _relay5 = Output::new(p.RELAY_5, Level::Low);
    let _relay6 = Output::new(p.RELAY_6, Level::Low);
    let _relay7 = Output::new(p.RELAY_7, Level::Low);

    // Safety critical things go on core 1
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE_1_STACK) },
        move || {
            let executor_1 = EXECUTOR_1.init(Executor::new(usize::MAX as *mut ()));
            let spawner = executor_1.spawner();

            unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

            // TODO
            // unwrap!(spawner.spawn(fuck_about_with_relays(r.relays)));
            unwrap!(spawner.spawn(measure_dat_pwm(r.flow_sensor)));

            #[cfg(feature = "test-panic-on-core-1")]
            unwrap!(spawner.spawn(dummy_panic()));

            loop {
                let before = Instant::now().as_ticks();
                cortex_m::asm::wfe();
                let after = Instant::now().as_ticks();
                SLEEP_TICKS_CORE_1.fetch_add(after - before, Ordering::Relaxed);
                if !PANIC_HALT.load(Ordering::Relaxed) {
                    unsafe { executor_1.poll() };
                }
            }
        },
    );

    // Everything else goes on core 0
    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    let spawner = executor_0.spawner();

    // TODO

    // CPU usage reporting
    unwrap!(spawner.spawn(report_cpu_usage()));

    #[cfg(feature = "test-panic-on-core-0")]
    unwrap!(spawner.spawn(dummy_panic()));

    loop {
        let before = Instant::now().as_ticks();
        cortex_m::asm::wfe();
        let after = Instant::now().as_ticks();
        SLEEP_TICKS_CORE_0.fetch_add(after - before, Ordering::Relaxed);
        if !PANIC_HALT.load(Ordering::Relaxed) {
            unsafe { executor_0.poll() };
        }
    }
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_millis(600));

    loop {
        watchdog.feed();
        onboard_led.toggle();
        Timer::after_millis(500).await;
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

#[embassy_executor::task]
async fn dummy_panic() {
    embassy_time::Timer::after_secs(5).await;
    panic!("oh dear, how sad. nevermind...");
}

#[embassy_executor::task]
async fn measure_dat_pwm(r: FlowSensorResources) {
    let cfg: embassy_rp::pwm::Config = Default::default();
    let pwm = embassy_rp::pwm::Pwm::new_input(
        r.pwm,
        r.pin,
        Pull::Down,
        embassy_rp::pwm::InputMode::RisingEdge,
        cfg,
    );

    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        info!("Input frequency: {} Hz", pwm.counter());
        pwm.set_counter(0);
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn fuck_about_with_relays(r: RelayOutputResources) {
    let mut fan = Output::new(r.fan, Level::Low);
    let mut compressor = Output::new(r.compressor, Level::Low);
    let mut stirrer = Output::new(r.stirrer, Level::Low);
    let mut pump = Output::new(r.pump, Level::Low);

    let mut ticker = Ticker::every(Duration::from_secs(5));

    loop {
        fan.toggle();
        ticker.next().await;
        fan.toggle();

        compressor.toggle();
        ticker.next().await;
        compressor.toggle();

        stirrer.toggle();
        ticker.next().await;
        stirrer.toggle();

        pump.toggle();
        ticker.next().await;
        pump.toggle();
    }
}
