#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod hal;
// mod fequency_counter;
mod sensors;
mod telemetry;

use atmega_hal::prelude::*;
use hoshiguma_foundational_data::satori::Status;
use one_wire_bus::OneWire;

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

    let machine_enable = pins.machine_enable.into_output();

    let mut coolant_level_sensor = {
        let top = pins.rj45_pin5.into_pull_up_input();
        let bottom = pins.rj45_pin6.into_pull_up_input();
        sensors::CoolantLevelSensor::new(top, bottom)
    };

    let coolant_flow_sensor = pins.rj45_pin3.into_pull_up_input();
    let coolant_pump_speed_sensor = pins.rj45_pin4.into_pull_up_input();

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

    loop {
        let count_0 = avr_device::interrupt::free(|_cs| {
            let count = COUNT_0.load(Ordering::SeqCst);
            COUNT_0.store(0, Ordering::SeqCst);
            count
        });

        let count_2 = avr_device::interrupt::free(|_cs| {
            let count = COUNT_2.load(Ordering::SeqCst);
            COUNT_2.store(0, Ordering::SeqCst);
            count
        });

        let coolant_level = coolant_level_sensor.read();
        let temperature = temperature_sensors.read();

        let status = Status {
            temperature,
            coolant_level,
            // TODO
            coolant_pump_rpm: 0.0,
            coolant_flow_rate: 0.0,
            potential_problems: heapless::Vec::new(),
            problems: heapless::Vec::new(),
        };

        telemetry::status(&mut serial, iteration_id, &status);

        // TODO
        // machine_enable.toggle();

        led.toggle();
        hal::Delay::new().delay_ms(250u16);

        iteration_id = iteration_id.wrapping_add(1);
    }
}

use core::sync::atomic::{AtomicU8, Ordering};

static COUNT_0: AtomicU8 = AtomicU8::new(0);
static COUNT_2: AtomicU8 = AtomicU8::new(0);

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT0() {
    let mut count = COUNT_0.load(Ordering::SeqCst);
    count += 1;
    COUNT_0.store(count, Ordering::SeqCst);
}

#[avr_device::interrupt(atmega328p)]
#[allow(non_snake_case)]
fn PCINT2() {
    let mut count = COUNT_2.load(Ordering::SeqCst);
    count += 1;
    COUNT_2.store(count, Ordering::SeqCst);
}
