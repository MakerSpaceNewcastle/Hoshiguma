#![no_std]
#![no_main]

mod changed;
mod devices;
mod io_helpers;
mod logic;
mod maybe_timer;
#[cfg(feature = "telemetry")]
mod telemetry;

use defmt::{info, unwrap};
use defmt_rtt as _;
use embassy_executor::{Executor, Spawner};
use embassy_rp::{
    gpio::{Input, Level, Output, OutputOpenDrain, Pull},
    multicore::{spawn_core1, Stack},
    watchdog::Watchdog,
};
use embassy_time::{Duration, Timer};
use one_wire_bus::OneWire;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use static_cell::StaticCell;

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use devices::{
        laser_enable::LaserEnableState, machine_enable::MachineEnableState,
        status_lamp::StatusLampSetting,
    };

    let p = unsafe { embassy_rp::Peripherals::steal() };

    // Disable the machine and laser
    let mut laser_enable = init_laser_enable!(p);
    let mut machine_enable = init_machine_enable!(p);
    laser_enable.set(LaserEnableState::Inhibited);
    machine_enable.set(MachineEnableState::Inhibited);

    // Report the panic
    #[cfg(feature = "telemetry")]
    {
        let mut uart = init_telemetry_uart!(p);
        crate::telemetry::report_panic(&mut uart, info);
    }

    // Set the status lamp to something distinctive
    let mut status_lamp = init_status_lamp!(p);
    status_lamp.set(StatusLampSetting {
        red: true,
        amber: true,
        green: true,
    });

    // Blink the on-board LED pretty fast
    let mut led = Output::new(p.PIN_25, Level::Low);
    loop {
        led.toggle();
        embassy_time::block_for(Duration::from_millis(50));
    }
}

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let onboard_led = Output::new(p.PIN_25, Level::Low);

    // Unused IO
    let _in0 = Input::new(p.PIN_15, Pull::Down);
    let _in1 = Input::new(p.PIN_14, Pull::Down);
    let _in2 = Input::new(p.PIN_13, Pull::Down);
    let _relay5 = Output::new(p.PIN_19, Level::Low);

    // Digital IO
    let status_lamp = init_status_lamp!(p);
    let machine_power_detector = init_machine_power_detector!(p);
    let chassis_intrusion_detector = init_chassis_intrusion_detector!(p);
    let air_assist_demand_detector = init_air_assist_demand_detector!(p);
    let air_assist_pump = init_air_assist_pump!(p);
    let machine_run_detector = init_machine_run_detector!(p);
    let fume_extraction_mode_switch = init_fume_extraction_mode_switch!(p);
    let fume_extraction_fan = init_fume_extraction_fan!(p);
    let coolant_resevoir_level_sensor = init_coolant_resevoir_level_sensor!(p);
    let laser_enable = init_laser_enable!(p);
    let machine_enable = init_machine_enable!(p);

    // Temperature sensors
    let mut onewire_bus = {
        let pin = OutputOpenDrain::new(p.PIN_22, Level::Low);
        OneWire::new(pin).unwrap()
    };

    for device_address in onewire_bus.devices(false, &mut embassy_time::Delay) {
        let device_address = device_address.unwrap();
        info!("Found one wire device at address: {}", device_address.0);
    }

    let mut watchdog = Watchdog::new(p.WATCHDOG);
    watchdog.start(Duration::from_millis(550));

    #[cfg(feature = "telemetry")]
    let mut telemetry_uart = init_telemetry_uart!(p);

    #[cfg(feature = "telemetry")]
    crate::telemetry::report_boot(&mut telemetry_uart).await;

    // Safety critical things go on core 1
    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                unwrap!(spawner.spawn(watchdog_feed_task(watchdog, onboard_led)));

                unwrap!(spawner.spawn(devices::machine_power_detector::task(
                    machine_power_detector,
                )));
                unwrap!(spawner.spawn(devices::digital_inputs::task(
                    chassis_intrusion_detector,
                    air_assist_demand_detector,
                    machine_run_detector,
                    fume_extraction_mode_switch,
                    coolant_resevoir_level_sensor,
                )));

                // State monitor tasks
                unwrap!(spawner.spawn(logic::safety::monitor::chassis_intrusion::task()));

                // State monitor observation and alarm tasks
                unwrap!(spawner.spawn(logic::safety::alarms::monitor_observation_task()));
                unwrap!(spawner.spawn(logic::safety::lockout::alarm_evaluation_task()));

                // Machine operation permission control tasks
                unwrap!(spawner.spawn(logic::safety::lockout::machine_lockout_task()));
                unwrap!(spawner.spawn(devices::laser_enable::task(laser_enable)));
                unwrap!(spawner.spawn(devices::machine_enable::task(machine_enable)));

                // Fume extraction control tasks
                unwrap!(spawner.spawn(logic::fume_extraction::task()));
                unwrap!(spawner.spawn(devices::fume_extraction_fan::task(fume_extraction_fan)));
            });
        },
    );

    // Everything else goes on core 0
    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        unwrap!(spawner.spawn(logic::status_lamp::task()));
        unwrap!(spawner.spawn(devices::status_lamp::task(status_lamp)));

        unwrap!(spawner.spawn(devices::temperature_sensors::task(onewire_bus)));

        // State monitor tasks
        unwrap!(spawner.spawn(logic::safety::monitor::power::task()));
        unwrap!(spawner.spawn(logic::safety::monitor::coolant_level::task()));
        unwrap!(spawner.spawn(logic::safety::monitor::temperatures::task()));

        // Air assist control tasks
        unwrap!(spawner.spawn(devices::air_assist::pump_task(air_assist_pump)));
        unwrap!(spawner.spawn(logic::air_assist::task()));

        // Telemetry reporting tasks
        #[cfg(feature = "telemetry")]
        crate::telemetry::spawn(spawner, telemetry_uart);
    });
}

#[embassy_executor::task]
async fn watchdog_feed_task(mut watchdog: Watchdog, mut led: Output<'static>) {
    loop {
        watchdog.feed();
        led.toggle();
        Timer::after_millis(500).await;
    }
}
