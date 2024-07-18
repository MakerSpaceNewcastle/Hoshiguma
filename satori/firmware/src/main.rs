#![no_std]
#![no_main]

mod rules;
mod sensors;

use crate::rules::RuleEvaluationContext;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, OutputOpenDrain, Pull};
use embassy_time::{Duration, Instant, Ticker, Timer};
use embedded_hal::digital::{OutputPin, PinState};
use heapless::Vec;
use hoshiguma_foundational_data::satori::{ObservedState, Status};
use one_wire_bus::OneWire;
#[cfg(feature = "panic-probe")]
use panic_probe as _;

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    info!("Program start");

    let p = embassy_rp::init(Default::default());

    let mut led = Output::new(p.PIN_25, Level::Low);

    // TODO: serial

    // TODO
    let mut machine_enable = Output::new(p.PIN_10, Level::Low);

    let mut coolant_level_sensor = {
        // TODO
        let top = Input::new(p.PIN_11, Pull::Up);
        // TODO
        let bottom = Input::new(p.PIN_12, Pull::Up);
        sensors::CoolantLevelSensor::new(top, bottom)
    };

    let mut onewire_bus = {
        let pin = OutputOpenDrain::new(p.PIN_13, Level::Low);
        OneWire::new(pin).unwrap()
    };

    for device_address in onewire_bus.devices(false, &mut embassy_time::Delay) {
        let device_address = device_address.unwrap();
        info!("Found one wire device at address: {:?}", device_address.0);
    }

    let mut temperature_sensors = crate::sensors::TemperatureSensors::new(onewire_bus, embassy_time::Delay);

    let mut iteration_id: u32 = 0;
    let mut last_potential_problems = Vec::new();

    let mut ticky = Ticker::every(Duration::from_hz(4));

    loop {
        ticky.next().await;
        let now = Instant::now().as_millis() as u32;
        info!("{} ms - {}", now, iteration_id);

        // TODO
        let coolant_pump_rpm = 0.0;

        // TODO
        let coolant_flow_rate = 0.0;

        info!("tempread start");
        let temperature = temperature_sensors.read();
        info!("tempread done");
        // let temperature = Default::default();
        let coolant_level = coolant_level_sensor.read();

        info!("{}C", temperature.electronics_bay);

        let observed = ObservedState {
            temperature,
            coolant_level,
            coolant_pump_rpm,
            coolant_flow_rate,
        };

        let mut potential_problems = Vec::new();
        let mut problems = Vec::new();

        crate::rules::evaluate(RuleEvaluationContext {
            state: &observed,
            now,
            last_potential_problems: &last_potential_problems,
            potential_problems: &mut potential_problems,
            problems: &mut problems,
        });

        let status = Status {
            observed,
            potential_problems,
            problems,
        };

        // Allow the machine to operate when there are no problems, otherwise disable it
        machine_enable
            .set_state(match status.problems.is_empty() {
                true => PinState::High,
                false => PinState::Low,
            })
            .unwrap();

        // TODO
        // telemetry::status(&mut serial, now, iteration_id, &status);

        led.toggle();

        iteration_id = iteration_id.wrapping_add(1);
        last_potential_problems = status.potential_problems;
    }
}

// #[panic_handler]
// fn panic(info: &core::panic::PanicInfo) -> ! {
//     avr_device::interrupt::disable();

//     let dp = unsafe { atmega_hal::Peripherals::steal() };
//     let pins = hal::Pins::with_mcu_pins(atmega_hal::pins!(dp));

//     // Disable machine
//     let mut machine_enable = pins.machine_enable.into_output();
//     machine_enable.set_low();

//     // Report panic over serial
//     let mut serial = serial!(dp, pins, 57600);
//     serial.write_byte(0);
//     telemetry::panic(&mut serial, info);

//     // Blink LED rapidly
//     let mut led = pins.led.into_output();
//     loop {
//         led.toggle();
//         hal::Delay::new().delay_ms(50u16);
//     }
// }
