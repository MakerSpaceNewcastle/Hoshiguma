#![no_std]
#![no_main]

mod changed;
mod devices;
mod logic;
mod maybe_timer;
mod polled_input;
mod telemetry;
mod trace;

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
use embassy_time::{Duration, Instant, Timer};
use git_version::git_version;
use hoshiguma_protocol::types::{BootReason, SystemInformation};
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::peripherals::{self, PicoPlc};
use portable_atomic::AtomicBool;
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    onewire: OnewireResources {
        pin: ONEWIRE,
    },
    status_lamp: StatusLampResources {
        red: RELAY_0,
        amber: RELAY_1,
        green: RELAY_2,
    },
    machine_power_detect: MachinePowerDetectResources {
        detect: IN_7,
    },
    chassis_intrusion_detect: ChassisIntrusionDetectResources {
        detect: IN_6,
    },
    air_assist_demand_detect: AirAssistDemandDetectResources {
        detect: IN_4,
    },
    machine_run_detect: MachineRunDetectResources {
        detect: IN_3,
    },
    fume_extraction_mode_switch: FumeExtractionModeSwitchResources {
        switch: IN_5,
    },
    coolant_resevoir_level_sensor: CoolantResevoirLevelSensorResources {
        empty: IN_1,
        low: IN_2,
    },
    air_assist_pump: AirAssistPumpResources {
        relay: RELAY_6,
    },
    fume_extraction_fan: FumeExtractionFanResources {
        relay: RELAY_7,
    },
    laser_enable: LaserEnableResources {
        relay: RELAY_4,
    },
    machine_enable: MachineEnableResources {
        relay: RELAY_3,
    },
    telemetry: TelemetryResources {
        uart: UART0,
        tx_pin: IO_0,
        rx_pin: IO_1,
    },
    cooler: CoolerCommunicationResources {
        uart: UART1,
        tx_pin: IO_4,
        rx_pin: IO_5,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    use crate::devices::{
        laser_enable::LaserEnableOutput, machine_enable::MachineEnableOutput,
        status_lamp::StatusLampOutput,
    };

    // Flag the panic, indicating that executors should stop scheduling work
    PANIC_HALT.store(true, Ordering::Relaxed);

    let p = unsafe { PicoPlc::steal() };
    let r = split_resources!(p);

    // Disable the machine and laser
    let mut laser_enable = LaserEnableOutput::new(r.laser_enable);
    laser_enable.set_panic();
    let mut machine_enable = MachineEnableOutput::new(r.machine_enable);
    machine_enable.set_panic();

    // Set the status lamp to something distinctive
    let mut status_lamp = StatusLampOutput::new(r.status_lamp);
    status_lamp.set_panic();

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed();

        // Keep setting the enable and status lamp outputs.
        // Not strictly needed, as no other tasks should be using the outputs at this point, but
        // here for belt and braces.
        laser_enable.set_panic();
        machine_enable.set_panic();
        status_lamp.set_panic();

        // Blink the on-board LED pretty fast
        led.toggle();

        embassy_time::block_for(Duration::from_millis(50));
    }
}

static mut CORE_1_STACK: Stack<4096> = Stack::new();

static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR_1: StaticCell<Executor> = StaticCell::new();

static PANIC_HALT: AtomicBool = AtomicBool::new(false);

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = PicoPlc::default();
    let r = split_resources!(p);

    info!("Version: {}", git_version!());

    // Unused IO
    let _in0 = Input::new(p.IN_0, Pull::Down);
    let _relay5 = Output::new(p.RELAY_5, Level::Low);

    // Safety critical things go on core 1
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE_1_STACK) },
        move || {
            let executor_1 = EXECUTOR_1.init(Executor::new(usize::MAX as *mut ()));
            trace::name_executor(executor_1.id() as u32, "core 1");
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
            unwrap!(spawner.spawn(logic::safety::monitor::observation_task()));
            unwrap!(spawner.spawn(logic::safety::lockout::alarm_evaluation_task()));

            // Machine operation permission control tasks
            unwrap!(spawner.spawn(logic::safety::lockout::machine_lockout_task()));
            unwrap!(spawner.spawn(devices::laser_enable::task(r.laser_enable)));
            unwrap!(spawner.spawn(devices::machine_enable::task(r.machine_enable)));

            #[cfg(feature = "test-panic-on-core-1")]
            unwrap!(spawner.spawn(dummy_panic()));

            loop {
                cortex_m::asm::wfe();
                if !PANIC_HALT.load(Ordering::Relaxed) {
                    unsafe { executor_1.poll() };
                }
            }
        },
    );

    // Everything else goes on core 0
    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    trace::name_executor(executor_0.id() as u32, "core 0");
    let spawner = executor_0.spawner();

    unwrap!(spawner.spawn(logic::status_lamp::task()));
    unwrap!(spawner.spawn(devices::status_lamp::task(r.status_lamp)));

    unwrap!(spawner.spawn(devices::coolant_resevoir_level_sensor::task(
        r.coolant_resevoir_level_sensor
    )));
    unwrap!(spawner.spawn(devices::temperature_sensors::task(r.onewire)));
    unwrap!(spawner.spawn(devices::cooler::task(r.cooler)));

    // State monitor tasks
    unwrap!(spawner.spawn(logic::safety::monitor::power::task()));
    unwrap!(spawner.spawn(logic::safety::monitor::coolant_level::task()));
    unwrap!(spawner.spawn(logic::safety::monitor::temperatures::task()));

    // Air assist control tasks
    unwrap!(spawner.spawn(devices::air_assist_demand_detector::task(
        r.air_assist_demand_detect
    )));
    unwrap!(spawner.spawn(devices::air_assist_pump::task(r.air_assist_pump)));
    unwrap!(spawner.spawn(logic::air_assist::task()));

    // Fume extraction control tasks
    unwrap!(spawner.spawn(devices::fume_extraction_mode_switch::task(
        r.fume_extraction_mode_switch
    )));
    unwrap!(spawner.spawn(devices::fume_extraction_fan::task(r.fume_extraction_fan)));
    unwrap!(spawner.spawn(logic::fume_extraction::task()));

    // Telemetry reporting
    unwrap!(spawner.spawn(telemetry::task(r.telemetry)));

    // Task reporting
    unwrap!(spawner.spawn(trace::task()));

    #[cfg(feature = "test-panic-on-core-0")]
    unwrap!(spawner.spawn(dummy_panic()));

    loop {
        cortex_m::asm::wfe();
        if !PANIC_HALT.load(Ordering::Relaxed) {
            unsafe { executor_0.poll() };
        }
    }
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    trace::name_task("wdt feed").await;

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
