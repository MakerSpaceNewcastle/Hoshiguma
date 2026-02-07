#![no_std]
#![no_main]

mod devices;
mod machine;
mod rpc;

use assign_resources::assign_resources;
use defmt::info;
use defmt_rtt as _;
use devices::{
    compressor::Compressor, coolant_flow_sensor::CoolantFlowSensor, coolant_pump::CoolantPump,
    coolant_reservoir_level_sensor::CoolantReservoirLevelSensor, radiator_fan::RadiatorFan,
    temperature_sensors::TemperatureSensors,
};
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use hoshiguma_core::types::BootReason;
use machine::Machine;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::{
    embassy_rp::{
        gpio::{Input, Level, Output, Pull},
        watchdog::Watchdog,
    },
    peripherals::{self, Peri, PicoPlc},
};

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
    coolant_reservoir_level: CoolantReservoirLevelSensorResources {
        low: IN_1,
    },
    compressor: CompressorResources {
        relay: RELAY_0,
    },
    coolant_pump: CoolantPumpResources {
        relay: RELAY_1,
    },
    radiator_fan: RadiatorFanResources {
        relay: RELAY_2,
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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = PicoPlc::default();
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    // Unused IO
    let _in2 = Input::new(p.IN_2, Pull::Down);
    let _in3 = Input::new(p.IN_3, Pull::Down);
    let _in4 = Input::new(p.IN_4, Pull::Down);
    let _in5 = Input::new(p.IN_5, Pull::Down);
    let _in6 = Input::new(p.IN_6, Pull::Down);
    let _in7 = Input::new(p.IN_7, Pull::Down);
    let _relay3 = Output::new(p.RELAY_3, Level::Low);
    let _relay4 = Output::new(p.RELAY_4, Level::Low);
    let _relay5 = Output::new(p.RELAY_5, Level::Low);
    let _relay6 = Output::new(p.RELAY_6, Level::Low);
    let _relay7 = Output::new(p.RELAY_7, Level::Low);

    // Outputs
    let coolant_pump = CoolantPump::new(r.coolant_pump);
    let compressor = Compressor::new(r.compressor);
    let radiator_fan = RadiatorFan::new(r.radiator_fan);

    // Inputs
    let coolant_reservoir_level_sensor =
        CoolantReservoirLevelSensor::new(r.coolant_reservoir_level);
    let coolant_flow_sensor = CoolantFlowSensor::new(&spawner, r.flow_sensor);
    let temperature_sensors = TemperatureSensors::new(&spawner, r.onewire);

    let machine = Machine {
        coolant_pump,
        compressor,
        radiator_fan,
        coolant_reservoir_level_sensor,
        coolant_flow_sensor,
        temperature_sensors,
    };

    spawner.must_spawn(watchdog_feed_task(r.status));

    spawner.must_spawn(rpc::task(r.communication, machine));

    #[cfg(feature = "test-panic-on-core-0")]
    spawner.must_spawn(dummy_panic());
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
