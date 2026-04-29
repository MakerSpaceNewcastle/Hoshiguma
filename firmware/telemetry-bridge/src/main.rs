#![no_std]
#![no_main]

mod api;
mod buttons;
mod network;
mod self_telemetry;
mod telegraf_buffer;
mod telemetry_tx;
mod wall_time;

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
use panic_probe as _;
use portable_atomic as _;

assign_resources::assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    ethernet_internal: Ethernet1Resources {
        miso: PIN_16,
        mosi: PIN_19,
        clk: PIN_18,
        spi: SPI0,
        tx_dma: DMA_CH0,
        rx_dma: DMA_CH1,
        cs_pin: PIN_17,
        int_pin: PIN_21,
        rst_pin: PIN_20,
    },
    ethernet_external: Ethernet2Resources {
        miso: PIN_8,
        mosi: PIN_11,
        clk: PIN_10,
        spi: SPI1,
        tx_dma: DMA_CH2,
        rx_dma: DMA_CH3,
        cs_pin: PIN_9,
        int_pin: PIN_13,
        rst_pin: PIN_12,
    },
    display: DisplayResources {
        i2c: I2C1,
        sda: PIN_14,
        scl: PIN_15,
    },
    buttons: ButtonResources {
        user_1: PIN_22,
        user_2: PIN_26,
        user_3: PIN_28,
    },
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    buttons::init(r.buttons, spawner);

    let net_stack_internal = network::init_ethernet_1(r.ethernet_internal, spawner).await;
    let net_stack_external = network::init_ethernet_2(r.ethernet_external, spawner).await;

    for idx in 0..api::NUM_LISTENERS {
        spawner.spawn(api::listen_task(net_stack_internal, idx).unwrap());
    }

    info!("Waiting for DHCP");
    net_stack_external.wait_config_up().await;
    let address = net_stack_external.config_v4().unwrap().address.address();
    info!("IP address: {}", address);

    spawner.spawn(wall_time::ntp_task(net_stack_external).unwrap());
    spawner.spawn(self_telemetry::task().unwrap());
    spawner.spawn(telemetry_tx::task(net_stack_external).unwrap());
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_secs(5));

    let mut led = Output::new(r.led, Level::Low);

    loop {
        // Flash LED steadily to indicate normal operation
        watchdog.feed(Duration::from_secs(2));
        led.toggle();

        Timer::after_secs(1).await;
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
