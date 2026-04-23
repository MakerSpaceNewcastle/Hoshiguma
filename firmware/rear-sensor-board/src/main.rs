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
        led: PIN_25,
    },
    ethernet: EthernetResources {
        miso: PIN_16,
        mosi: PIN_19,
        clk: PIN_18,
        spi: SPI0,
        tx_dma: DMA_CH0,
        rx_dma: DMA_CH1,
        cs: PIN_17,
        int: PIN_21,
        reset: PIN_20,
    },
    onewire: OnewireResources {
        pio: PIO1,
        pin: PIN_28,
    },
    status_light: StatusLightResources {
        red: PIN_13,
        amber: PIN_14,
        green: PIN_15,
    },
    sdp810: Sdp810Resources {
        i2c: I2C1,
        sda: PIN_2,
        scl: PIN_3,
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

    static AIRFLOW_COMM: StaticCell<[devices::airflow_sensor::Channel; NUM_LISTENERS]> =
        StaticCell::new();
    let airflow_comm = AIRFLOW_COMM.init(Default::default());
    let airflow_comm_b = airflow_comm.each_ref().map(|comm| comm.side_b());
    spawner.spawn(devices::airflow_sensor::task(r.sdp810, airflow_comm_b).unwrap());

    static STATUS_LIGHT_COMM: StaticCell<[devices::status_light::Channel; NUM_LISTENERS]> =
        StaticCell::new();
    let status_light_comm = STATUS_LIGHT_COMM.init(Default::default());
    let status_light_comm_b = status_light_comm.each_ref().map(|comm| comm.side_b());
    spawner.spawn(devices::status_light::task(r.status_light, status_light_comm_b).unwrap());

    static TEMPERATURES_COMM: StaticCell<[devices::temperature_sensors::Channel; NUM_LISTENERS]> =
        StaticCell::new();
    let temperatures_comm = TEMPERATURES_COMM.init(Default::default());
    let temperatures_comm_b = temperatures_comm.each_ref().map(|comm| comm.side_b());
    spawner.spawn(devices::temperature_sensors::task(r.onewire, temperatures_comm_b).unwrap());

    let mut comm = heapless::Vec::new();
    for i in 0..network::NUM_LISTENERS {
        if comm
            .push(DeviceCommunicator {
                airflow: airflow_comm[i].side_a(),
                status_light: status_light_comm[i].side_a(),
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
    airflow: devices::airflow_sensor::TheirChannelSide,
    status_light: devices::status_light::TheirChannelSide,
    temperatures: devices::temperature_sensors::TheirChannelSide,
}

static COMM_GOOD_INDICATOR: Channel<CriticalSectionRawMutex, (), 8> = Channel::new();

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) -> ! {
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
