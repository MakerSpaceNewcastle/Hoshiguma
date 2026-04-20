#![no_std]
#![no_main]

mod devices;
mod network;

use assign_resources::assign_resources;
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    peripherals,
    watchdog::Watchdog,
};
use embassy_time::{Duration, Timer};
use hoshiguma_api::BootReason;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use portable_atomic as _;
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_19,
    },
    onewire: OnewireResources {
        pio: PIO1,
        pin: PIN_28,
    },
    coolant_rate_sensors: CoolantRateSensorResources {
        flow_pwm: PWM_SLICE2,
        flow_pin: PIN_5, // Input 1

        return_pwm: PWM_SLICE3,
        return_pin: PIN_7, // Input 3
    },
    compressor: CompressorResources {
        relay: PIN_13, // Relay 6
    },
    coolant_pump: CoolantPumpResources {
        relay: PIN_14, // Relay 7
    },
    radiator_fan: RadiatorFanResources {
        relay: PIN_15, // Relay 8
    },
    ethernet: EthernetResources {
        pio: PIO0,
        mosi: PIN_23,
        miso: PIN_22,
        sck: PIN_21,
        tx_dma: DMA_CH0,
        rx_dma: DMA_CH1,
        cs: PIN_20,
        int: PIN_24,
        reset: PIN_25,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let p = unsafe { embassy_rp::Peripherals::steal() };
    let r = split_resources!(p);

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed(Duration::from_millis(100));

        // Blink the on-board LED pretty fast
        led.toggle();

        embassy_time::block_for(Duration::from_millis(50));
    }
}

// TODO: communication watchdog

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    static COMPRESSOR_COMM: StaticCell<devices::compressor::Channel> = StaticCell::new();
    let compressor_comm = COMPRESSOR_COMM.init(Default::default());
    spawner.spawn(devices::compressor::task(r.compressor, compressor_comm.side_b()).unwrap());

    static COOLANT_PUMP_COMM: StaticCell<devices::coolant_pump::Channel> = StaticCell::new();
    let coolant_pump_comm = COOLANT_PUMP_COMM.init(Default::default());
    spawner.spawn(devices::coolant_pump::task(r.coolant_pump, coolant_pump_comm.side_b()).unwrap());

    static COOLANT_FLOW_RATE_COMM: StaticCell<devices::coolant_rate_sensors::Channel> =
        StaticCell::new();
    let coolant_flow_rate_comm = COOLANT_FLOW_RATE_COMM.init(Default::default());
    static COOLANT_RETURN_RATE_COMM: StaticCell<devices::coolant_rate_sensors::Channel> =
        StaticCell::new();
    let coolant_return_rate_comm = COOLANT_RETURN_RATE_COMM.init(Default::default());
    devices::coolant_rate_sensors::start(
        spawner,
        r.coolant_rate_sensors,
        coolant_flow_rate_comm.side_b(),
        coolant_return_rate_comm.side_b(),
    );

    static RADIATOR_FAN_COMM: StaticCell<devices::radiator_fan::Channel> = StaticCell::new();
    let radiator_fan_comm = RADIATOR_FAN_COMM.init(Default::default());
    spawner.spawn(devices::radiator_fan::task(r.radiator_fan, radiator_fan_comm.side_b()).unwrap());

    static TEMPERATURES_COMM: StaticCell<devices::temperature_sensors::Channel> = StaticCell::new();
    let temperatures_comm = TEMPERATURES_COMM.init(Default::default());
    spawner
        .spawn(devices::temperature_sensors::task(r.onewire, temperatures_comm.side_b()).unwrap());

    let mut machine_control = heapless::Vec::new();
    for _ in 0..network::NUM_LISTENERS {
        if machine_control
            .push(MachineControl {
                compressor: compressor_comm.side_a(),
                coolant_pump: coolant_pump_comm.side_a(),
                coolant_flow_rate: coolant_flow_rate_comm.side_a(),
                coolant_return_rate: coolant_return_rate_comm.side_a(),
                radiator_fan: radiator_fan_comm.side_a(),
                temperatures: temperatures_comm.side_a(),
            })
            .is_err()
        {
            panic!();
        }
    }
    spawner.spawn(network::task(spawner, r.ethernet, machine_control).unwrap());

    spawner.spawn(watchdog_feed_task(r.status).unwrap());

    #[cfg(feature = "test-panic-on-core-0")]
    spawner.spawn(dummy_panic().unwrap());

    // TODO
    let mut temp_sensors_comm = temperatures_comm.side_a();
    loop {
        temp_sensors_comm
            .to_you
            .publish(devices::temperature_sensors::Request)
            .await;
        let res = temp_sensors_comm.to_me.next_message_pure().await;
        info!("Temperature sensors: {:?}", res);
        Timer::after_secs(10).await;
    }
}

struct MachineControl {
    compressor: devices::compressor::TheirChannelSide,
    coolant_pump: devices::coolant_pump::TheirChannelSide,
    coolant_flow_rate: devices::coolant_rate_sensors::TheirChannelSide,
    coolant_return_rate: devices::coolant_rate_sensors::TheirChannelSide,
    radiator_fan: devices::radiator_fan::TheirChannelSide,
    temperatures: devices::temperature_sensors::TheirChannelSide,
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    let watchdog_timeout = Duration::from_millis(600);
    watchdog.start(watchdog_timeout);

    loop {
        watchdog.feed(watchdog_timeout);
        onboard_led.toggle();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn dummy_panic() {
    embassy_time::Timer::after_secs(5).await;
    panic!("oh dear, how sad. nevermind...");
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
