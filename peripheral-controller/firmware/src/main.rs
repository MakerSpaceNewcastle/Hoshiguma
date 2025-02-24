#![no_std]
#![no_main]

mod changed;
mod devices;
mod io_helpers;
mod logic;
mod maybe_timer;
mod telemetry;

use assign_resources::assign_resources;
use core::sync::atomic::Ordering;
use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::raw::Executor;
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    multicore::{spawn_core1, Stack},
    peripherals,
    watchdog::Watchdog,
};
use embassy_time::{Duration, Instant, Ticker, Timer};
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use portable_atomic::AtomicU64;
use static_cell::StaticCell;
use telemetry::TelemetryUart;

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use devices::{
        laser_enable::{LaserEnable, LaserEnableState},
        machine_enable::{MachineEnable, MachineEnableState},
        status_lamp::{StatusLamp, StatusLampSetting},
    };

    let p = unsafe { embassy_rp::Peripherals::steal() };
    let r = split_resources!(p);

    // Disable the machine and laser
    let mut laser_enable: LaserEnable = r.laser_enable.into();
    let mut machine_enable: MachineEnable = r.machine_enable.into();
    laser_enable.set(LaserEnableState::Inhibited);
    machine_enable.set(MachineEnableState::Inhibited);

    // Report the panic
    let mut uart: TelemetryUart = r.telemetry.into();
    crate::telemetry::report_panic(&mut uart, info);

    // Set the status lamp to something distinctive
    let mut status_lamp: StatusLamp = r.status_lamp.into();
    status_lamp.set(StatusLampSetting {
        red: true,
        amber: true,
        green: true,
    });

    // Blink the on-board LED pretty fast
    let mut led = Output::new(r.status.led, Level::Low);
    loop {
        led.toggle();
        embassy_time::block_for(Duration::from_millis(50));
    }
}

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    onewire: OnewireResources {
        pin: PIN_22,
    },
    status_lamp: StatusLampResources {
        red: PIN_7, // Relay 0
        amber: PIN_6, // Relay 1
        green: PIN_16, // Relay 2
    },
    machine_power_detect: MachinePowerDetectResources {
        detect: PIN_8, // Input 7
    },
    chassis_intrusion_detect: ChassisIntrusionDetectResources {
        detect: PIN_9, // Input 4
    },
    air_assist_demand_detect: AirAssistDemandDetectResources {
        detect: PIN_11, // Input 4
    },
    machine_run_detect: MachineRunDetectResources {
        detect: PIN_12, // Input 3
    },
    fume_extraction_mode_switch: FumeExtractionModeSwitchResources {
        switch: PIN_10, // Input 5
    },
    coolant_resevoir_level_sensor: CoolantResevoirLevelSensorResources {
        empty: PIN_4, // Level shifted IO 4
        low: PIN_5, // Level shifted IO 5
    },
    air_assist_pump: AirAssistPumpResources {
        relay: PIN_20, // Relay 6
    },
    fume_extraction_fan: FumeExtractionFanResources {
        relay: PIN_21, // Relay 7
    },
    laser_enable: LaserEnableResources {
        relay: PIN_18, // Relay 4
    },
    machine_enable: MachineEnableResources {
        relay: PIN_17, // Relay 3
    },
    telemetry: TelemetryResources {
        tx_pin: PIN_0,
        uart: UART0,
        dma_ch: DMA_CH0,
    },
}

static mut CORE_1_STACK: Stack<4096> = Stack::new();

static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR_1: StaticCell<Executor> = StaticCell::new();

static SLEEP_TICKS_CORE_0: AtomicU64 = AtomicU64::new(0);
static SLEEP_TICKS_CORE_1: AtomicU64 = AtomicU64::new(0);

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    // Unused IO
    let _in0 = Input::new(p.PIN_15, Pull::Down);
    let _in1 = Input::new(p.PIN_14, Pull::Down);
    let _in2 = Input::new(p.PIN_13, Pull::Down);
    let _relay5 = Output::new(p.PIN_19, Level::Low);

    let mut telemetry_uart: TelemetryUart = r.telemetry.into();
    crate::telemetry::report_boot(&mut telemetry_uart);

    // Safety critical things go on core 1
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE_1_STACK) },
        move || {
            let executor_1 = EXECUTOR_1.init(Executor::new(usize::MAX as *mut ()));
            let spawner = executor_1.spawner();

            unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

            unwrap!(spawner.spawn(devices::machine_power_detector::task(
                r.machine_power_detect
            )));
            unwrap!(spawner.spawn(devices::machine_run_detector::task(r.machine_run_detect)));
            unwrap!(spawner.spawn(devices::chassis_intrusion_detector::task(
                r.chassis_intrusion_detect
            )));

            // State monitor tasks
            unwrap!(spawner.spawn(logic::safety::monitor::chassis_intrusion::task()));

            // State monitor observation and alarm tasks
            unwrap!(spawner.spawn(logic::safety::alarms::monitor_observation_task()));
            unwrap!(spawner.spawn(logic::safety::lockout::alarm_evaluation_task()));

            // Machine operation permission control tasks
            unwrap!(spawner.spawn(logic::safety::lockout::machine_lockout_task()));
            unwrap!(spawner.spawn(devices::laser_enable::task(r.laser_enable)));
            unwrap!(spawner.spawn(devices::machine_enable::task(r.machine_enable)));

            // Fume extraction control tasks
            unwrap!(spawner.spawn(devices::fume_extraction_mode_switch::task(
                r.fume_extraction_mode_switch
            )));
            unwrap!(spawner.spawn(devices::fume_extraction_fan::task(r.fume_extraction_fan)));
            unwrap!(spawner.spawn(logic::fume_extraction::task()));

            loop {
                let before = Instant::now().as_ticks();
                cortex_m::asm::wfe();
                let after = Instant::now().as_ticks();
                SLEEP_TICKS_CORE_1.fetch_add(after - before, Ordering::Relaxed);
                unsafe { executor_1.poll() };
            }
        },
    );

    // Everything else goes on core 0
    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    let spawner = executor_0.spawner();

    unwrap!(spawner.spawn(logic::status_lamp::task()));
    unwrap!(spawner.spawn(devices::status_lamp::task(r.status_lamp)));

    unwrap!(spawner.spawn(devices::coolant_resevoir_level_sensor::task(
        r.coolant_resevoir_level_sensor
    )));
    unwrap!(spawner.spawn(devices::temperature_sensors::task(r.onewire)));

    // State monitor tasks
    unwrap!(spawner.spawn(logic::safety::monitor::power::task()));
    unwrap!(spawner.spawn(logic::safety::monitor::coolant_level::task()));
    unwrap!(spawner.spawn(logic::safety::monitor::temperatures::task()));

    // Air assist control tasks
    unwrap!(spawner.spawn(devices::air_assist::demand_task(r.air_assist_demand_detect)));
    unwrap!(spawner.spawn(devices::air_assist::pump_task(r.air_assist_pump)));
    unwrap!(spawner.spawn(logic::air_assist::task()));

    // Telemetry reporting tasks
    crate::telemetry::spawn(spawner, telemetry_uart);

    // CPU usage reporting
    unwrap!(spawner.spawn(report_cpu_usage()));

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
    watchdog.start(Duration::from_millis(550));

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
