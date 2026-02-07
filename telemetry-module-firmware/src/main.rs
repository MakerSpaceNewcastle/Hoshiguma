#![no_std]
#![no_main]

mod buttons;
mod display;
mod network;
mod rpc_server;
mod self_telemetry;
mod usb_serial;

use crate::buttons::{UI_INPUTS, UiEvent};
use defmt::info;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_futures::select::{Either, select};
use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    peripherals,
    watchdog::Watchdog,
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
};
use embassy_time::{Duration, Timer};
use heapless::String;
use hoshiguma_core::types::BootReason;
use panic_probe as _;
use portable_atomic as _;

assign_resources::assign_resources! {
    ethernet: EthernetResources {
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
    display: DisplayResources {
        mosi_pin: PIN_11,
        clk_pin: PIN_10,
        dc_pin: PIN_13,
        reset_pin: PIN_12,
        backlight_pin: PIN_14,
        spi: SPI1,
        backlight_pwm: PWM_SLICE7,
    }
    rs485_uart_1: Rs485Uart1Resources {
        tx_pin: PIN_0,
        rx_pin: PIN_1,
        uart: UART0,
    }
    buttons: ButtonResources {
        a_pin: PIN_6,
        b_pin: PIN_7,
        c_pin: PIN_8,
    }
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    }
    usb: UsbResources {
        usb: USB,
    }
}

pub(crate) static TELEMETRY_TX: PubSubChannel<CriticalSectionRawMutex, String<128>, 32, 2, 2> =
    PubSubChannel::new();

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    spawner.must_spawn(watchdog_feed_task(r.status));
    spawner.must_spawn(crate::usb_serial::task(r.usb, spawner));
    let net_stack = crate::network::init(r.ethernet, spawner).await;
    spawner.must_spawn(crate::self_telemetry::task());
    spawner.must_spawn(crate::rpc_server::task(r.rs485_uart_1, net_stack));
    spawner.must_spawn(crate::buttons::task(r.buttons));
    spawner.must_spawn(crate::display::task(r.display));

    #[cfg(feature = "test-panic-on-core-0")]
    spawner.must_spawn(dummy_panic());
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: crate::StatusResources) {
    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_secs(5));

    let mut led = Output::new(r.led, Level::Low);

    let mut ui_event_rx = UI_INPUTS.subscriber().unwrap();

    loop {
        match select(Timer::after_secs(1), ui_event_rx.next_message()).await {
            Either::First(_) => {
                // Flash LED steadily to indicate normal operation
                watchdog.feed();
                led.toggle();
            }
            Either::Second(WaitResult::Lagged(_)) => unreachable!(),
            Either::Second(WaitResult::Message(UiEvent::ButtonPushedForALongTime)) => {
                info!("Triggering reboot!");
                watchdog.trigger_reset();
            }
            _ => {}
        }
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
