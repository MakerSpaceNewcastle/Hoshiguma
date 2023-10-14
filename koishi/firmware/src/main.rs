#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod checked_update;
mod hal;
mod io;
mod logic;
#[cfg(feature = "reporting_postcard")]
mod reporting;
mod unwrap_simple;

use crate::{
    checked_update::CheckedUpdate,
    io::{
        inputs::ReadInputs,
        outputs::{Outputs, WriteOutputs},
    },
    logic::{
        air_assist::AirAssistStatus, extraction::ExtractionStatus, machine::MachineStatus,
        StatusUpdate,
    },
};
use atmega_hal::prelude::*;

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();

    let dp = unsafe { atmega_hal::Peripherals::steal() };
    let pins = hal::Pins::with_mcu_pins(atmega_hal::pins!(dp));

    #[cfg(feature = "reporting_postcard")]
    {
        let mut serial = serial!(dp, pins, 57600);
        serial.write_byte(0u8);
        reporting::panic(&mut serial, _info);
    }

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

    #[cfg(feature = "reporting_postcard")]
    let mut serial = serial!(dp, pins, 57600);
    #[cfg(feature = "reporting_postcard")]
    reporting::boot(&mut serial);

    #[cfg(feature = "devkit")]
    let inputs = gpio_debug_inputs!(pins);
    #[cfg(not(feature = "devkit"))]
    let inputs = gpio_isolated_inputs!(pins);

    let mut outputs = gpio_relay_outputs!(pins);

    let mut st_inputs = CheckedUpdate::default();
    let mut machine_status = CheckedUpdate::new(MachineStatus::default());
    let mut extraction_status = CheckedUpdate::new(ExtractionStatus::default());
    let mut air_assist_status = CheckedUpdate::new(AirAssistStatus::default());
    let mut st_outputs = CheckedUpdate::default();

    let mut iteration_id: u32 = 0;

    loop {
        let time = crate::hal::millis();

        if st_inputs.store(inputs.read()) {
            #[cfg(feature = "reporting_postcard")]
            reporting::status(&mut serial, iteration_id, st_inputs.get());
        }

        if machine_status.store(machine_status.get().update(time, st_inputs.get())) {
            #[cfg(feature = "reporting_postcard")]
            reporting::status(&mut serial, iteration_id, machine_status.get());
        }

        if extraction_status.store(extraction_status.get().update(time, st_inputs.get())) {
            #[cfg(feature = "reporting_postcard")]
            reporting::status(&mut serial, iteration_id, extraction_status.get());
        }

        if air_assist_status.store(air_assist_status.get().update(time, st_inputs.get())) {
            #[cfg(feature = "reporting_postcard")]
            reporting::status(&mut serial, iteration_id, air_assist_status.get());
        }

        if st_outputs.store(Outputs::new(
            machine_status.get(),
            extraction_status.get(),
            air_assist_status.get(),
        )) {
            #[cfg(feature = "reporting_postcard")]
            reporting::status(&mut serial, iteration_id, st_outputs.get());
        }

        outputs.write(st_outputs.get());

        iteration_id = iteration_id.wrapping_add(1);
    }
}
