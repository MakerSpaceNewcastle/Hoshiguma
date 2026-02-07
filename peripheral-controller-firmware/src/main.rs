#![no_std]
#![no_main]

mod changed;
mod cli;
mod devices;
mod logic;
mod maybe_timer;
mod polled_input;
mod self_telemetry;
mod telemetry;
#[cfg(feature = "trace")]
mod trace;

use assign_resources::assign_resources;
use core::sync::atomic::Ordering;
use defmt::{info, warn};
use defmt_rtt as _;
use embassy_executor::raw::Executor;
use embassy_time::{Duration, Instant, Ticker, Timer};
use hoshiguma_core::types::BootReason;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::{
    embassy_rp::{
        gpio::{Input, Level, Output, Pull},
        multicore::{Stack, spawn_core1},
        watchdog::Watchdog,
    },
    peripherals::{self, Peri, PicoPlc},
};
use portable_atomic::AtomicBool;
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    usb: UsbResources {
        usb: USB,
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
    access_control: AccessControlResources {
        detect: IN_2,
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
    machine_power: MachinePowerResources {
        relay: RELAY_5,
    },
    telemetry: TelemetryResources {
        uart: UART0,
        tx_pin: IO_0,
        rx_pin: IO_1,
    },
    accessories_bus: AccessoriesCommunicationResources {
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

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    // Unused IO
    let _in0 = Input::new(p.IN_0, Pull::Down);
    let _in1 = Input::new(p.IN_1, Pull::Down);

    // Safety critical things go on core 1
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE_1_STACK) },
        move || {
            let executor_1 = EXECUTOR_1.init(Executor::new(usize::MAX as *mut ()));
            #[cfg(feature = "trace")]
            trace::identify_core_1_executor(executor_1.id() as u32);
            let spawner = executor_1.spawner();

            spawner.must_spawn(watchdog_feed_task(r.status));

            spawner.must_spawn(devices::machine_power_detector::task(
                r.machine_power_detect,
            ));
            spawner.must_spawn(devices::machine_run_detector::task(r.machine_run_detect));
            spawner.must_spawn(devices::chassis_intrusion_detector::task(
                r.chassis_intrusion_detect,
            ));

            // State monitor tasks
            spawner.must_spawn(logic::safety::monitor::chassis_intrusion::task());

            // State monitor observation and alarm tasks
            spawner.must_spawn(logic::safety::monitor::observation_task());
            spawner.must_spawn(logic::safety::lockout::alarm_evaluation_task());

            // Machine operation permission control tasks
            spawner.must_spawn(logic::safety::lockout::machine_lockout_task());
            spawner.must_spawn(devices::laser_enable::task(r.laser_enable));
            spawner.must_spawn(devices::machine_enable::task(r.machine_enable));

            #[cfg(feature = "test-panic-on-core-1")]
            spawner.must_spawn(dummy_panic());

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
    #[cfg(feature = "trace")]
    trace::identify_core_0_executor(executor_0.id() as u32);
    let spawner = executor_0.spawner();

    // Telemetry reporting
    spawner.must_spawn(self_telemetry::task());
    spawner.must_spawn(telemetry::task(r.telemetry));

    spawner.must_spawn(cli::task(r.usb, spawner));

    spawner.must_spawn(logic::status_lamp::task());
    spawner.must_spawn(devices::status_lamp::task(r.status_lamp));

    spawner.must_spawn(devices::temperature_sensors::task(r.onewire));
    spawner.must_spawn(devices::accessories::task(r.accessories_bus));

    spawner.must_spawn(quick_and_dirty_machine_power_for_new_access_control_task(
        r.access_control,
        r.machine_power,
    ));

    // State monitor tasks
    spawner.must_spawn(logic::safety::monitor::power::task());
    spawner.must_spawn(logic::safety::monitor::coolant_flow::task());
    spawner.must_spawn(logic::safety::monitor::coolant_level::task());
    spawner.must_spawn(logic::safety::monitor::extraction_airflow::task());
    spawner.must_spawn(logic::safety::monitor::temperatures_a::task());
    spawner.must_spawn(logic::safety::monitor::temperatures_b::task());

    // Air assist control tasks
    spawner.must_spawn(devices::air_assist_demand_detector::task(
        r.air_assist_demand_detect,
    ));
    spawner.must_spawn(devices::air_assist_pump::task(r.air_assist_pump));
    spawner.must_spawn(logic::air_assist::task());

    // Fume extraction control tasks
    spawner.must_spawn(devices::fume_extraction_mode_switch::task(
        r.fume_extraction_mode_switch,
    ));
    spawner.must_spawn(devices::fume_extraction_fan::task(r.fume_extraction_fan));
    spawner.must_spawn(logic::fume_extraction::task());

    // Cooler control tasks
    spawner.must_spawn(logic::cooling::control::task());
    spawner.must_spawn(logic::cooling::demand::task());

    // Task reporting
    #[cfg(feature = "trace")]
    spawner.must_spawn(trace::task());

    #[cfg(feature = "test-panic-on-core-0")]
    spawner.must_spawn(dummy_panic());

    loop {
        cortex_m::asm::wfe();
        if !PANIC_HALT.load(Ordering::Relaxed) {
            unsafe { executor_0.poll() };
        }
    }
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    #[cfg(feature = "trace")]
    trace::name_task("wdt feed").await;

    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_millis(500));

    let mut feed_ticker = Ticker::every(Duration::from_millis(100));

    loop {
        let start = Instant::now();

        // Blink LED
        onboard_led.toggle();

        // at 1 Hz
        for _ in 0..10 {
            watchdog.feed();
            feed_ticker.next().await;
        }

        let end = Instant::now();
        let duration = end - start;
        if duration > Duration::from_millis(1050) {
            warn!(
                "WDT feed loop took a suspicious amount of time: {}",
                duration
            );
        }
    }
}

#[embassy_executor::task]
async fn dummy_panic() {
    embassy_time::Timer::after_secs(5).await;
    panic!("oh dear, how sad. nevermind...");
}

fn boot_reason() -> BootReason {
    let reason = pico_plc_bsp::embassy_rp::pac::WATCHDOG.reason().read();

    if reason.force() {
        BootReason::WatchdogForced
    } else if reason.timer() {
        BootReason::WatchdogTimeout
    } else {
        BootReason::Normal
    }
}

#[embassy_executor::task]
async fn quick_and_dirty_machine_power_for_new_access_control_task(
    ir: AccessControlResources,
    or: MachinePowerResources,
) {
    let access_control = Input::new(ir.detect, Pull::Down);
    let mut machine_power = Output::new(or.relay, Level::Low);

    loop {
        let level = access_control.get_level();
        machine_power.set_level(level);

        Timer::after_millis(200).await;
    }
}
