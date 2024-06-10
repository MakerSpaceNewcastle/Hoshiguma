#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod checked_update;
mod hal;
mod unwrap_simple;
// mod status;

use atmega_hal::prelude::*;
use one_wire_bus::OneWire;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();

    let dp = unsafe { atmega_hal::Peripherals::steal() };
    let pins = hal::Pins::with_mcu_pins(atmega_hal::pins!(dp));

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
    serial.write_str("feck arse drink\n").unwrap();

    let _d7 = pins.d7.into_pull_up_input();
    let _d9 = pins.d9.into_pull_up_input();

    // Enable the PCINT0 and PCINT2 interrupts
    // See datasheet: 12.2.4 PCICR - Pin Change Interrupt Control Register
    dp.EXINT.pcicr.write(|w| unsafe { w.bits(0b00000101) });

    // Enable pin change interrupts on PCINT23 which is pin PD7 (= d7)
    // See datasheet: 13.3.3 Alternate Functions of Port D
    dp.EXINT.pcmsk2.write(|w| w.bits(0b10000000));

    // Enable pin change interrupts on PCINT1 which is pin PB1 (= d9)
    // See datasheet: 13.3.1 Alternate Functions of Port B
    dp.EXINT.pcmsk0.write(|w| w.bits(0b00000010));

    let mut delay = hal::Delay::new();

    let one_wire_pin = pins.d4.into_opendrain();
    let mut one_wire_bus = OneWire::new(one_wire_pin).unwrap();
    for device_address in one_wire_bus.devices(false, &mut delay) {
        let device_address = device_address.unwrap();

        ufmt::uwriteln!(serial, "Found device at address {}", device_address.0).unwrap();
    }

    let mut led = pins.led.into_output();

    loop {
        let _time = crate::hal::millis();

        let count_0 = avr_device::interrupt::free(|_cs| {
            let count = COUNT_0.load(Ordering::SeqCst);
            COUNT_0.store(0, Ordering::SeqCst);
            count
        });
        ufmt::uwrite!(serial, "count 0 = {}\n", count_0).unwrap();

        let count_2 = avr_device::interrupt::free(|_cs| {
            let count = COUNT_2.load(Ordering::SeqCst);
            COUNT_2.store(0, Ordering::SeqCst);
            count
        });
        ufmt::uwrite!(serial, "count 2 = {}\n", count_2).unwrap();

        // TODO
        led.toggle();
        hal::Delay::new().delay_ms(500u16);
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
