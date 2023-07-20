#![no_std]
#![no_main]
#![feature(abi_avr_interrupt)]

mod checked_update;
mod hal;
mod io;
mod logic;
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
fn panic(info: &core::panic::PanicInfo) -> ! {
    avr_device::interrupt::disable();

    let dp = unsafe { atmega_hal::Peripherals::steal() };
    let pins = hal::Pins::with_mcu_pins(atmega_hal::pins!(dp));

    let mut serial = serial!(dp, pins, 57600);

    ufmt::uwriteln!(&mut serial, "Firmware panic!\r").void_unwrap();
    if let Some(loc) = info.location() {
        ufmt::uwriteln!(
            &mut serial,
            "  At {}:{}:{}\r",
            loc.file(),
            loc.line(),
            loc.column(),
        )
        .void_unwrap();
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

    #[cfg(not(feature = "simulator"))]
    let mut serial = serial!(dp, pins, 57600);
    #[cfg(not(feature = "simulator"))]
    ufmt::uwriteln!(&mut serial, "Hello, world!").void_unwrap();

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

    loop {
        let time = crate::hal::millis();

        if st_inputs.store(inputs.read()) {
            #[cfg(not(feature = "simulator"))]
            ufmt::uwriteln!(&mut serial, "[{}] {:#?}", time, st_inputs.get()).void_unwrap();
        }

        if machine_status.store(machine_status.get().update(time, st_inputs.get())) {
            #[cfg(not(feature = "simulator"))]
            ufmt::uwriteln!(
                &mut serial,
                "[{}] Machine status: {:#?}",
                time,
                machine_status.get()
            )
            .void_unwrap();
        }

        if extraction_status.store(extraction_status.get().update(time, st_inputs.get())) {
            #[cfg(not(feature = "simulator"))]
            ufmt::uwriteln!(
                &mut serial,
                "[{}] Extraction status: {:#?}",
                time,
                extraction_status.get()
            )
            .void_unwrap();
        }

        if air_assist_status.store(air_assist_status.get().update(time, st_inputs.get())) {
            #[cfg(not(feature = "simulator"))]
            ufmt::uwriteln!(
                &mut serial,
                "[{}] Air assist status: {:#?}",
                time,
                air_assist_status.get()
            )
            .void_unwrap();
        }

        if st_outputs.store(Outputs::new(
            machine_status.get(),
            extraction_status.get(),
            air_assist_status.get(),
        )) {
            #[cfg(not(feature = "simulator"))]
            ufmt::uwriteln!(&mut serial, "[{}] {:#?}", time, st_outputs.get()).void_unwrap();
        }

        outputs.write(st_outputs.get());
    }
}
