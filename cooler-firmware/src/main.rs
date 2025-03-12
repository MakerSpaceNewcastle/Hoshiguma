#![no_std]
#![no_main]

mod devices;
mod polled_input;
mod rpc;

use assign_resources::assign_resources;
use core::sync::atomic::Ordering;
use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::raw::Executor;
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    watchdog::Watchdog,
};
use embassy_time::{Duration, Instant, Ticker, Timer};
use git_version::git_version;
use hoshiguma_protocol::types::{BootReason, SystemInformation};
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::peripherals::{self, PicoPlc};
use portable_atomic::AtomicU64;
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    onewire: OnewireResources {
        pin: ONEWIRE,
    },
    flow_sensor: FlowSensorResources {
        pwm: PWM_SLICE7,
        pin: IN_0,
    },
    heat_exchanger_level: HeatExchangerLevelSensorResources {
        low: IN_1,
    },
    header_tank_level: HeaderTankLevelSensorResources {
        empty: IN_2,
        low : IN_3,
    },
    compressor: CompressorResources {
        relay: RELAY_0,
    },
    stirrer: StirrerResources {
        relay: RELAY_1,
    },
    radiator_fan: RadiatorFanResources {
        relay: RELAY_2,
    },
    coolant_pump: CoolantPumpResources {
        relay: RELAY_3,
    },
    communication: ControlCommunicationResources {
        uart: UART0,
        tx_pin: IO_0,
        rx_pin: IO_1,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let p = unsafe { PicoPlc::steal() };
    let r = split_resources!(p);

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

static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
static SLEEP_TICKS_CORE_0: AtomicU64 = AtomicU64::new(0);

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

    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    let spawner = executor_0.spawner();

    unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

    // Devices
    unwrap!(spawner.spawn(devices::radiator_fan::task(r.radiator_fan)));
    unwrap!(spawner.spawn(devices::compressor::task(r.compressor)));
    unwrap!(spawner.spawn(devices::stirrer::task(r.stirrer)));
    unwrap!(spawner.spawn(devices::coolant_pump::task(r.coolant_pump)));
    unwrap!(spawner.spawn(devices::temperature_sensors::task(r.onewire)));
    unwrap!(spawner.spawn(devices::coolant_flow_sensor::task(r.flow_sensor)));
    unwrap!(spawner.spawn(devices::heat_exchanger_level_sensor::task(
        r.heat_exchanger_level
    )));
    unwrap!(spawner.spawn(devices::header_tank_level_sensor::task(r.header_tank_level)));

    // RPC/telemetry/control
    unwrap!(spawner.spawn(rpc::task(r.communication)));

    // CPU usage reporting
    unwrap!(spawner.spawn(report_cpu_usage()));

    #[cfg(feature = "test-panic-on-core-0")]
    unwrap!(spawner.spawn(dummy_panic()));

    loop {
        let before = Instant::now().as_ticks();
        cortex_m::asm::wfe();
        let after = Instant::now().as_ticks();
        SLEEP_TICKS_CORE_0.fetch_add(after - before, Ordering::Relaxed);
        unsafe { executor_0.poll() };
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

    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        ticker.next().await;

        let current_tick = Instant::now().as_ticks();
        let tick_difference = (current_tick - previous_tick) as f32;

        let current_sleep_tick_core_0 = SLEEP_TICKS_CORE_0.load(Ordering::Relaxed);

        let calc_cpu_usage = |current_sleep_tick: u64, previous_sleep_tick: u64| -> f32 {
            let sleep_tick_difference = (current_sleep_tick - previous_sleep_tick) as f32;
            1f32 - sleep_tick_difference / tick_difference
        };

        let usage_core_0 = calc_cpu_usage(current_sleep_tick_core_0, previous_sleep_tick_core_0);

        previous_tick = current_tick;
        previous_sleep_tick_core_0 = current_sleep_tick_core_0;

        info!("Usage: core 0 = {}", usage_core_0);
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
