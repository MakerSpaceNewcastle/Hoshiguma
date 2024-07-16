#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod frequency_counter;
mod hal;
mod rules;
mod sensors;
mod telemetry;

use atmega_hal::prelude::*;
use embedded_hal::digital::{OutputPin, PinState};
use heapless::Vec;
use hoshiguma_foundational_data::satori::{ObservedState, Status};
use one_wire_bus::OneWire;
use rules::RuleEvaluationContext;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();

    let dp = unsafe { atmega_hal::Peripherals::steal() };
    let pins = hal::Pins::with_mcu_pins(atmega_hal::pins!(dp));

    // Disable machine
    let mut machine_enable = pins.machine_enable.into_output();
    machine_enable.set_low();

    // Report panic over serial
    let mut serial = serial!(dp, pins, 57600);
    serial.write_byte(0);
    telemetry::panic(&mut serial, info);

    // Blink LED rapidly
    let mut led = pins.led.into_output();
    loop {
        led.toggle();
        hal::Delay::new().delay_ms(50u16);
    }
}

#[avr_device::entry]
fn main() -> ! {
    let dp = atmega_hal::Peripherals::take().unwrap();
    let pins = hal::Pins::with_mcu_pins(atmega_hal::pins!(dp));

    hal::millis_init(dp.TC0);
    unsafe { avr_device::interrupt::enable() };

    let mut serial = serial!(dp, pins, 57600);
    telemetry::boot(&mut serial);

    let mut machine_enable = pins.machine_enable.into_output();

    let mut coolant_level_sensor = {
        let top = pins.rj45_pin5.into_pull_up_input();
        let bottom = pins.rj45_pin6.into_pull_up_input();
        sensors::CoolantLevelSensor::new(top, bottom)
    };

    let _coolant_flow_sensor = pins.rj45_pin3.into_pull_up_input();
    let _coolant_pump_speed_sensor = pins.rj45_pin4.into_pull_up_input();

    // Enable the PCINT0 and PCINT2 interrupts
    // See datasheet: 12.2.4 PCICR - Pin Change Interrupt Control Register
    dp.EXINT.pcicr.write(|w| unsafe { w.bits(0b00000101) });

    // Enable pin change interrupts on PCINT23 which is pin PD7 (= d7)
    // See datasheet: 13.3.3 Alternate Functions of Port D
    dp.EXINT.pcmsk2.write(|w| w.bits(0b10000000));

    // Enable pin change interrupts on PCINT0 which is pin PB0 (= d8)
    // See datasheet: 13.3.1 Alternate Functions of Port B
    dp.EXINT.pcmsk0.write(|w| w.bits(0b00000001));

    let mut delay = hal::Delay::new();

    let mut onewire_bus = {
        let pin = pins.rj45_pin2.into_opendrain();
        OneWire::new(pin).unwrap()
    };

    for device_address in onewire_bus.devices(false, &mut delay) {
        let device_address = device_address.unwrap();
        telemetry::found_onewire_device(&mut serial, device_address.0);
    }

    let mut temperature_sensors = crate::sensors::TemperatureSensors::new(onewire_bus, delay);

    let mut led = pins.led.into_output();

    let mut iteration_id: u32 = 0;
    let mut last_potential_problems = Vec::new();

    loop {
        let now = crate::hal::millis();

        let count_0 = avr_device::interrupt::free(|_| unsafe {
            let count = COUNT_0;
            COUNT_0 = 0;
            count
        });
        // TODO
        let coolant_pump_rpm = count_0;

        let count_2 = avr_device::interrupt::free(|_| unsafe {
            let count = COUNT_2;
            COUNT_2 = 0;
            count
        });
        // TODO
        let coolant_flow_rate = count_2;

        let temperature = temperature_sensors.read();
        let coolant_level = coolant_level_sensor.read();

        let observed = ObservedState {
            temperature,
            coolant_level,
            coolant_pump_rpm,
            coolant_flow_rate,
        };

        let potential_problems = Vec::new();
        let problems = Vec::new();

        crate::rules::evaluate(&RuleEvaluationContext {
            state: &observed,
            now,
            last_potential_problems: &last_potential_problems,
            potential_problems: &potential_problems,
            problems: &problems,
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

        telemetry::status(&mut serial, now, iteration_id, &status);

        led.toggle();
        hal::Delay::new().delay_ms(250u16);

        iteration_id = iteration_id.wrapping_add(1);
        last_potential_problems = status.potential_problems;
    }
}

static mut COUNT_0: u32 = 0;
static mut COUNT_2: u32 = 0;

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT0() {
    avr_device::interrupt::free(|_| {
        unsafe {
            COUNT_0 = COUNT_0.saturating_add(1);
        };
    });
}

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT2() {
    avr_device::interrupt::free(|_| {
        unsafe {
            COUNT_2 = COUNT_2.saturating_add(1);
        };
    });
}
