#![no_std]
#![no_main]

mod changed;
mod devices;
mod io_helpers;
mod logic;
mod maybe_timer;
mod telemetry;

use assign_resources::assign_resources;
use defmt::unwrap;
use defmt_rtt as _;
use embassy_executor::Executor;
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    multicore::{spawn_core1, Stack},
    peripherals,
    watchdog::Watchdog,
};
use embassy_time::{Duration, Timer};
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use portable_atomic as _;
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

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

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
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

                unwrap!(spawner.spawn(devices::machine_power_detector::task(
                    r.machine_power_detect
                )));
                unwrap!(spawner.spawn(devices::digital_inputs::task(
                    r.chassis_intrusion_detect,
                    r.air_assist_demand_detect,
                    r.machine_run_detect,
                    r.fume_extraction_mode_switch,
                    r.coolant_resevoir_level_sensor,
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
                unwrap!(spawner.spawn(logic::fume_extraction::task()));
                unwrap!(spawner.spawn(devices::fume_extraction_fan::task(r.fume_extraction_fan)));
            });
        },
    );

    // Everything else goes on core 0
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        unwrap!(spawner.spawn(logic::status_lamp::task()));
        unwrap!(spawner.spawn(devices::status_lamp::task(r.status_lamp)));

        unwrap!(spawner.spawn(devices::temperature_sensors::task(r.onewire)));

        // State monitor tasks
        unwrap!(spawner.spawn(logic::safety::monitor::power::task()));
        unwrap!(spawner.spawn(logic::safety::monitor::coolant_level::task()));
        unwrap!(spawner.spawn(logic::safety::monitor::temperatures::task()));

        // Air assist control tasks
        unwrap!(spawner.spawn(devices::air_assist::pump_task(r.air_assist_pump)));
        unwrap!(spawner.spawn(logic::air_assist::task()));

        // Telemetry reporting tasks
        crate::telemetry::spawn(spawner, telemetry_uart);
    });
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
