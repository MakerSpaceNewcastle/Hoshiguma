#![no_std]
#![no_main]

mod rules;
mod sensors;

use crate::rules::RuleEvaluationContext;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::gpio::{Input, Level, Output, OutputOpenDrain, Pull};
use embassy_time::{Duration, Instant, Ticker};
use embedded_hal::digital::{OutputPin, PinState};
use heapless::Vec;
use hoshiguma_foundational_data::satori::{ObservedState, Status};
use one_wire_bus::OneWire;
#[cfg(feature = "panic-probe")]
use panic_probe as _;

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    use embassy_rp::Peripherals;
    use embedded_hal::delay::DelayNs;

    let p = unsafe { Peripherals::steal() };

    // Disable machine
    let mut machine_enable = Output::new(p.PIN_9, Level::Low);
    machine_enable.set_low();

    // Blink the on board LED pretty fast
    let mut led = Output::new(p.PIN_25, Level::Low);
    loop {
        led.toggle();
        embassy_time::Delay.delay_ms(50);
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut led = Output::new(p.PIN_25, Level::Low);

    let mut machine_enable = Output::new(p.PIN_9, Level::Low);

    let mut coolant_level_sensor = {
        let top = Input::new(p.PIN_12, Pull::Up);
        let bottom = Input::new(p.PIN_14, Pull::Up);
        sensors::CoolantLevelSensor::new(top, bottom)
    };

    let mut onewire_bus = {
        let pin = OutputOpenDrain::new(p.PIN_10, Level::Low);
        OneWire::new(pin).unwrap()
    };

    for device_address in onewire_bus.devices(false, &mut embassy_time::Delay) {
        let device_address = device_address.unwrap();
        info!("Found one wire device at address: {}", device_address.0);
    }

    let mut temperature_sensors =
        crate::sensors::TemperatureSensors::new(onewire_bus, embassy_time::Delay);

    let fc_p13 = embassy_rp::pwm::Pwm::new_input(
        p.PWM_SLICE6,
        p.PIN_13,
        Pull::Down,
        embassy_rp::pwm::InputMode::FallingEdge,
        embassy_rp::pwm::Config::default(),
    );

    let fc_p15 = embassy_rp::pwm::Pwm::new_input(
        p.PWM_SLICE7,
        p.PIN_15,
        Pull::Down,
        embassy_rp::pwm::InputMode::FallingEdge,
        embassy_rp::pwm::Config::default(),
    );

    let mut last_potential_problems = Vec::new();

    let mut tick = Ticker::every(Duration::from_secs(2));

    loop {
        tick.next().await;

        let now = Instant::now().as_millis() as u32;
        info!("{} ms", now);

        // TODO
        let count = fc_p13.counter();
        info!("fc p13 count: {}", count);
        fc_p13.set_counter(0);
        let coolant_pump_rpm = 0.0;

        // TODO
        let count = fc_p15.counter();
        info!("fc p15 count: {}", count);
        fc_p15.set_counter(0);
        let coolant_flow_rate = 0.0;

        let temperature = temperature_sensors.read();

        let coolant_level = coolant_level_sensor.read();

        // TODO
        info!("{} C", temperature.coolant_flow);
        info!("{} C", temperature.coolant_return);
        info!("{} C", temperature.coolant_resevoir_lower);
        info!("{} C", temperature.coolant_resevoir_upper);
        info!("{} C", temperature.coolant_pump);
        info!("{} C", temperature.room_ambient);
        info!("{} C", temperature.laser_bay);
        info!("{} C", temperature.electronics_bay);
        info!(
            "coolant level: {}",
            match coolant_level {
                Some(ref level) => match level {
                    hoshiguma_foundational_data::satori::CoolantLevel::Full => "full",
                    hoshiguma_foundational_data::satori::CoolantLevel::Low => "low",
                    hoshiguma_foundational_data::satori::CoolantLevel::CriticallyLow => "empty",
                },
                None => "unknown",
            }
        );

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

        led.toggle();

        last_potential_problems = status.potential_problems;
    }
}
