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
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use hoshiguma_api::BootReason;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use portable_atomic as _;
use static_cell::StaticCell;

use crate::network::NUM_LISTENERS;

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

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    static COMPRESSOR_COMM: StaticCell<[devices::compressor::Channel; NUM_LISTENERS]> =
        StaticCell::new();
    let compressor_comm = COMPRESSOR_COMM.init(Default::default());
    let compressor_comm_b = compressor_comm.each_ref().map(|comm| comm.side_b());
    spawner.spawn(devices::compressor::task(r.compressor, compressor_comm_b).unwrap());

    static COOLANT_PUMP_COMM: StaticCell<[devices::coolant_pump::Channel; NUM_LISTENERS]> =
        StaticCell::new();
    let coolant_pump_comm = COOLANT_PUMP_COMM.init(Default::default());
    let coolant_pump_comm_b = coolant_pump_comm.each_ref().map(|comm| comm.side_b());
    spawner.spawn(devices::coolant_pump::task(r.coolant_pump, coolant_pump_comm_b).unwrap());

    static COOLANT_FLOW_RATE_COMM: StaticCell<
        [devices::coolant_rate_sensors::Channel; NUM_LISTENERS],
    > = StaticCell::new();
    let coolant_flow_rate_comm = COOLANT_FLOW_RATE_COMM.init(Default::default());
    let coolant_flow_rate_comm_b = coolant_flow_rate_comm.each_ref().map(|comm| comm.side_b());
    static COOLANT_RETURN_RATE_COMM: StaticCell<
        [devices::coolant_rate_sensors::Channel; NUM_LISTENERS],
    > = StaticCell::new();
    let coolant_return_rate_comm = COOLANT_RETURN_RATE_COMM.init(Default::default());
    let coolant_return_rate_comm_b = coolant_return_rate_comm
        .each_ref()
        .map(|comm| comm.side_b());
    devices::coolant_rate_sensors::start(
        spawner,
        r.coolant_rate_sensors,
        coolant_flow_rate_comm_b,
        coolant_return_rate_comm_b,
    );

    static RADIATOR_FAN_COMM: StaticCell<[devices::radiator_fan::Channel; NUM_LISTENERS]> =
        StaticCell::new();
    let radiator_fan_comm = RADIATOR_FAN_COMM.init(Default::default());
    let radiator_fan_comm_b = radiator_fan_comm.each_ref().map(|comm| comm.side_b());
    spawner.spawn(devices::radiator_fan::task(r.radiator_fan, radiator_fan_comm_b).unwrap());

    static TEMPERATURES_COMM: StaticCell<[devices::temperature_sensors::Channel; NUM_LISTENERS]> =
        StaticCell::new();
    let temperatures_comm = TEMPERATURES_COMM.init(Default::default());
    let temperatures_comm_b = temperatures_comm.each_ref().map(|comm| comm.side_b());
    spawner.spawn(devices::temperature_sensors::task(r.onewire, temperatures_comm_b).unwrap());

    let mut comm = heapless::Vec::new();
    for i in 0..network::NUM_LISTENERS {
        if comm
            .push(DeviceCommunicator {
                compressor: compressor_comm[i].side_a(),
                coolant_pump: coolant_pump_comm[i].side_a(),
                coolant_flow_rate: coolant_flow_rate_comm[i].side_a(),
                coolant_return_rate: coolant_return_rate_comm[i].side_a(),
                radiator_fan: radiator_fan_comm[i].side_a(),
                temperatures: temperatures_comm[i].side_a(),
            })
            .is_err()
        {
            panic!();
        }
    }
    network::init(spawner, r.ethernet, comm).await;

    spawner.spawn(watchdog_feed_task(r.status).unwrap());
}

struct DeviceCommunicator {
    compressor: devices::compressor::TheirChannelSide,
    coolant_pump: devices::coolant_pump::TheirChannelSide,
    coolant_flow_rate: devices::coolant_rate_sensors::TheirChannelSide,
    coolant_return_rate: devices::coolant_rate_sensors::TheirChannelSide,
    radiator_fan: devices::radiator_fan::TheirChannelSide,
    temperatures: devices::temperature_sensors::TheirChannelSide,
}

static COMM_GOOD_INDICATOR: Channel<CriticalSectionRawMutex, (), 8> = Channel::new();

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_secs(5));

    loop {
        let _ = COMM_GOOD_INDICATOR.receive().await;

        watchdog.feed(Duration::from_secs(5));

        // Blink the LED
        onboard_led.set_high();
        Timer::after_millis(10).await;
        onboard_led.set_low();
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
