use crate::TelemetryResources;
#[cfg(not(feature = "panic-probe"))]
use core::panic::PanicInfo;
use defmt::{debug, error, unwrap};
use embassy_executor::Spawner;
use embassy_rp::{
    peripherals::UART0,
    uart::{Async, Config as UartConfig, UartTx},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Ticker};
use hoshiguma_protocol::{
    payload::{
        system::{Boot, BootReason, Info, SystemMessagePayload},
        Payload,
    },
    serial::TELEMETRY_BAUD,
    Message,
};

pub(crate) type TelemetryUart = UartTx<'static, UART0, Async>;

impl From<TelemetryResources> for TelemetryUart {
    fn from(r: TelemetryResources) -> Self {
        let mut config = UartConfig::default();
        config.baudrate = TELEMETRY_BAUD;

        embassy_rp::uart::UartTx::new(r.uart, r.tx_pin, r.dma_ch, config)
    }
}

async fn tx_message(uart: &mut TelemetryUart, msg: &Message) {
    match postcard::to_vec_cobs::<Message, 128>(msg) {
        Ok(data) => match uart.write(&data).await {
            Ok(_) => debug!("Sent telemetry message"),
            Err(_) => error!("Failed to write telemetry message to UART"),
        },
        Err(_) => error!("Failed to serialize telemetry message"),
    }
}

fn tx_message_blocking(uart: &mut TelemetryUart, msg: &Message) {
    match postcard::to_vec_cobs::<Message, 128>(msg) {
        Ok(data) => match uart.blocking_write(&data) {
            Ok(_) => debug!("Sent telemetry message"),
            Err(_) => error!("Failed to write telemetry message to UART"),
        },
        Err(_) => error!("Failed to serialize telemetry message"),
    }
}

fn new_message(payload: Payload) -> Message {
    hoshiguma_protocol::Message {
        millis_since_boot: embassy_time::Instant::now().as_millis(),
        payload,
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

pub(super) fn report_boot(uart: &mut TelemetryUart) {
    let msg = new_message(Payload::System(SystemMessagePayload::Boot(Boot {
        git_revision: git_version::git_version!().try_into().unwrap(),
        reason: boot_reason(),
    })));
    tx_message_blocking(uart, &msg);
}

#[cfg(not(feature = "panic-probe"))]
pub(super) fn report_panic(uart: &mut TelemetryUart, info: &PanicInfo<'_>) {
    let msg = new_message(Payload::System(SystemMessagePayload::Panic(info.into())));
    tx_message_blocking(uart, &msg);
}

pub(super) fn spawn(spawner: Spawner, uart: TelemetryUart) {
    unwrap!(spawner.spawn(telemetry_tx_task(uart)));
    unwrap!(spawner.spawn(heartbeat_task()));
}

static TELEMETRY_MESSAGES: Channel<CriticalSectionRawMutex, Message, 32> = Channel::new();

#[embassy_executor::task]
pub(super) async fn telemetry_tx_task(mut uart: TelemetryUart) {
    loop {
        let msg = TELEMETRY_MESSAGES.receive().await;
        tx_message(&mut uart, &msg).await;
    }
}

#[embassy_executor::task]
pub(super) async fn heartbeat_task() {
    let mut ticker = Ticker::every(Duration::from_secs(15));

    loop {
        ticker.next().await;

        let msg = new_message(Payload::System(SystemMessagePayload::Heartbeat(Info {
            git_revision: git_version::git_version!().try_into().unwrap(),
        })));
        TELEMETRY_MESSAGES.send(msg).await;
    }
}

pub(crate) async fn queue_telemetry_message(payload: Payload) {
    let msg = new_message(payload);
    TELEMETRY_MESSAGES.send(msg).await;
}
