#![no_std]
#![no_main]

mod devices;
mod machine;
mod rpc;

use assign_resources::assign_resources;
use defmt::{info, unwrap};
use defmt_rtt as _;
use devices::{
    compressor::Compressor, coolant_flow_sensor::CoolantFlowSensor, coolant_pump::CoolantPump,
    header_tank_level_sensor::HeaderTankLevelSensor,
    heat_exchanger_level_sensor::HeatExchangerLevelSensor, radiator_fan::RadiatorFan,
    stirrer::Stirrer, temperature_sensors::TemperatureSensors,
};
use embassy_executor::Spawner;
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    watchdog::Watchdog,
};
use embassy_time::{Duration, Instant, Timer};
use hoshiguma_protocol::types::{BootReason, SystemInformation};
use machine::Machine;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::peripherals::{self, PicoPlc};
use portable_atomic as _;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    onewire: OnewireResources {
        pin: ONEWIRE,
    },
    flow_sensor: FlowSensorResources {
        pwm: PWM_SLICE7,
        pin: IN_0,
    },
    heat_exchanger_level: HeatExchangerLevelSensorResources {
        low: IN_1,
    },
    header_tank_level: HeaderTankLevelSensorResources {
        empty: IN_3,
        low : IN_2,
    },
    compressor: CompressorResources {
        relay: RELAY_0,
    },
    stirrer: StirrerResources {
        relay: RELAY_1,
    },
    radiator_fan: RadiatorFanResources {
        relay: RELAY_2,
    },
    coolant_pump: CoolantPumpResources {
        relay: RELAY_3,
    },
    communication: ControlCommunicationResources {
        uart: UART0,
        tx_pin: IO_0,
        rx_pin: IO_1,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let p = unsafe { PicoPlc::steal() };
    let r = split_resources!(p);

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed();

        // Blink the on-board LED pretty fast
        led.toggle();

        embassy_time::block_for(Duration::from_millis(50));
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = PicoPlc::default();
    let r = split_resources!(p);

    info!("{}", system_information());

    // Unused IO
    let _in5 = Input::new(p.IN_5, Pull::Down);
    let _in6 = Input::new(p.IN_6, Pull::Down);
    let _in7 = Input::new(p.IN_7, Pull::Down);
    let _relay4 = Output::new(p.RELAY_4, Level::Low);
    let _relay5 = Output::new(p.RELAY_5, Level::Low);
    let _relay6 = Output::new(p.RELAY_6, Level::Low);
    let _relay7 = Output::new(p.RELAY_7, Level::Low);

    // Outputs
    let stirrer = Stirrer::new(r.stirrer);
    let coolant_pump = CoolantPump::new(r.coolant_pump);
    let compressor = Compressor::new(r.compressor);
    let radiator_fan = RadiatorFan::new(r.radiator_fan);

    // Inputs
    let header_tank_level = HeaderTankLevelSensor::new(r.header_tank_level);
    let heat_exchanger_level = HeatExchangerLevelSensor::new(r.heat_exchanger_level);
    let coolant_flow_sensor = CoolantFlowSensor::new(&spawner, r.flow_sensor);
    let temperature_sensors = TemperatureSensors::new(&spawner, r.onewire);

    let machine = Machine {
        stirrer,
        coolant_pump,
        compressor,
        radiator_fan,
        header_tank_level,
        heat_exchanger_level,
        coolant_flow_sensor,
        temperature_sensors,
    };

    unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

    unwrap!(spawner.spawn(rpc::task(r.communication, machine,)));

    #[cfg(feature = "test-panic-on-core-0")]
    unwrap!(spawner.spawn(dummy_panic()));
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_millis(600));

    loop {
        watchdog.feed();
        onboard_led.toggle();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn dummy_panic() {
    embassy_time::Timer::after_secs(5).await;
    panic!("oh dear, how sad. nevermind...");
}

fn system_information() -> SystemInformation {
    SystemInformation {
        git_revision: git_version::git_version!().try_into().unwrap(),
        last_boot_reason: boot_reason(),
        uptime_milliseconds: Instant::now().as_millis(),
    }
}

fn boot_reason() -> BootReason {
    let reason = embassy_rp::pac::WATCHDOG.reason().read();

    if reason.force() {
        BootReason::WatchdogForced
    } else if reason.timer() {
        BootReason::WatchdogTimeout
    } else {
        BootReason::Normal
    }
}
