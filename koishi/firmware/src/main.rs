#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod checked_update;
mod hal;
mod io;
mod logic;
#[cfg(feature = "telemetry")]
mod telemetry;
mod unwrap_simple;

use crate::{
    checked_update::CheckedUpdate,
    io::{
        inputs::ReadInputs,
        outputs::{OutputsExt, WriteOutputs},
    },
    logic::{air_assist::AirAssistStatusExt, extraction::ExtractionStatusExt, StatusUpdate},
};
use atmega_hal::prelude::*;
use hoshiguma_foundational_data::koishi::{
    AirAssistStatus, ExtractionStatus, MachineStatus, Outputs,
};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();

    let dp = unsafe { atmega_hal::Peripherals::steal() };
    let pins = hal::Pins::with_mcu_pins(atmega_hal::pins!(dp));

    #[cfg(feature = "telemetry")]
    {
        let mut serial = serial!(dp, pins, 57600);
        serial.write_byte(0);
        telemetry::panic(&mut serial, info);
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

    #[cfg(feature = "telemetry")]
    let mut serial = serial!(dp, pins, 57600);
    #[cfg(feature = "telemetry")]
    telemetry::boot(&mut serial);

    #[cfg(feature = "devkit")]
    let inputs = gpio_debug_inputs!(pins);
    #[cfg(not(feature = "devkit"))]
    let inputs = gpio_isolated_inputs!(pins);

    let mut outputs = gpio_relay_outputs!(pins);

    let mut st_inputs = CheckedUpdate::default();
    let mut machine_status = CheckedUpdate::new(MachineStatus::Idle);
    let mut extraction_status = CheckedUpdate::new(ExtractionStatus::default());
    let mut air_assist_status = CheckedUpdate::new(AirAssistStatus::default());
    let mut st_outputs = CheckedUpdate::default();

    let mut iteration_id: u32 = 0;

    loop {
        let time = crate::hal::millis();

        if st_inputs.store(inputs.read()) {
            #[cfg(feature = "telemetry")]
            telemetry::status(&mut serial, iteration_id, st_inputs.get());
        }

        if machine_status.store(machine_status.get().update(time, st_inputs.get())) {
            #[cfg(feature = "telemetry")]
            telemetry::status(&mut serial, iteration_id, machine_status.get());
        }

        if extraction_status.store(extraction_status.get().update(time, st_inputs.get())) {
            #[cfg(feature = "telemetry")]
            telemetry::status(&mut serial, iteration_id, extraction_status.get());
        }

        if air_assist_status.store(air_assist_status.get().update(time, st_inputs.get())) {
            #[cfg(feature = "telemetry")]
            telemetry::status(&mut serial, iteration_id, air_assist_status.get());
        }

        if st_outputs.store(Outputs::new(
            machine_status.get(),
            extraction_status.get(),
            air_assist_status.get(),
        )) {
            #[cfg(feature = "telemetry")]
            telemetry::status(&mut serial, iteration_id, st_outputs.get());
        }

        outputs.write(st_outputs.get());

        iteration_id = iteration_id.wrapping_add(1);
    }
}
