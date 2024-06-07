#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod checked_update;
mod hal;
mod unwrap_simple;

use atmega_hal::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();

    let dp = unsafe { atmega_hal::Peripherals::steal() };
    let pins = hal::Pins::with_mcu_pins(atmega_hal::pins!(dp));

    let mut led = pins.d13.into_output();
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
    serial.write_str("feck arse drink").unwrap();

    let mut led = pins.d13.into_output();

    loop {
        let _time = crate::hal::millis();

        // TODO
        led.toggle();
        hal::Delay::new().delay_ms(500u16);
    }
}
