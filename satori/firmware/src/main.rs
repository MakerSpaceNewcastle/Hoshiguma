#![no_std]
#![no_main]

mod rules;
mod sensors;

use crate::rules::RuleEvaluationContext;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{self as _, gpio::{Level, Output}};
use embassy_time::{Duration, Timer};
use embedded_hal::digital::{OutputPin, PinState, StatefulOutputPin};
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
    // let mut machine_enable = pins.gpio10.into_push_pull_output_in_state(PinState::Low);

    // let mut coolant_level_sensor = {
    //     // TODO
    //     let top = pins.gpio11.into_pull_up_input();
    //     // TODO
    //     let bottom = pins.gpio12.into_pull_up_input();
    //     sensors::CoolantLevelSensor::new(top, bottom)
    // };

    // let mut onewire_bus = {
    //     let pin = pins.gpio13.into_push_pull_output();
    //     OneWire::new(pin).unwrap()
    // };

    // for device_address in onewire_bus.devices(false, &mut delay) {
    //     let device_address = device_address.unwrap();
    //     info!("Found one wire device at address: {:?}", device_address.0);
    // }

    // let mut temperature_sensors = crate::sensors::TemperatureSensors::new(onewire_bus, delay);

    // let mut iteration_id: u32 = 0;
    // let mut last_potential_problems = Vec::new();

    loop {
        // TODO
        let now = 0;

        // TODO
        let coolant_pump_rpm = 0.0;

        // TODO
        let coolant_flow_rate = 0.0;

        // let temperature = temperature_sensors.read();
        // let coolant_level = coolant_level_sensor.read();

        // let observed = ObservedState {
        //     temperature,
        //     coolant_level,
        //     coolant_pump_rpm,
        //     coolant_flow_rate,
        // };

        // let mut potential_problems = Vec::new();
        // let mut problems = Vec::new();

        // crate::rules::evaluate(RuleEvaluationContext {
        //     state: &observed,
        //     now,
        //     last_potential_problems: &last_potential_problems,
        //     potential_problems: &mut potential_problems,
        //     problems: &mut problems,
        // });

        // let status = Status {
        //     observed,
        //     potential_problems,
        //     problems,
        // };

        // // Allow the machine to operate when there are no problems, otherwise disable it
        // machine_enable
        //     .set_state(match status.problems.is_empty() {
        //         true => PinState::High,
        //         false => PinState::Low,
        //     })
        //     .unwrap();

        // TODO
        // telemetry::status(&mut serial, now, iteration_id, &status);

        led.toggle();
        Timer::after(Duration::from_millis(250)).await;
        info!("doot");

        // iteration_id = iteration_id.wrapping_add(1);
        // last_potential_problems = status.potential_problems;
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
