use crate::{
    TelemetryResources,
    logic::safety::monitor::NEW_MONITOR_STATUS,
    self_telemetry::{DATA_POINT_TEMPLATE_ERRORS, DATA_POINTS_DISCARDED},
};
use chrono::{DateTime, TimeDelta, Utc};
use core::{sync::atomic::Ordering, time::Duration};
use defmt::{debug, error, info, unwrap, warn};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Instant, Timer};
use hoshiguma_core::{
    telemetry::StaticTelemetryDataPoint,
    telemetry_module::{
        StringRegistry,
        rpc::{Request, Response},
    },
    time::NtpSyncedTime,
    types::{MonitorKind, Severity},
};
use pico_plc_bsp::embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use static_cell::StaticCell;
use teeny_rpc::{
    RpcMessage,
    client::Client,
    transport::{Transport, embedded::EioTransport},
};

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(super) async fn task(r: TelemetryResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("telemetry").await;

    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buffer = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 32;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buffer = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = hoshiguma_core::telemetry_module::SERIAL_BAUD;

    let uart = BufferedUart::new(
        r.uart, r.tx_pin, r.rx_pin, Irqs, tx_buffer, rx_buffer, config,
    );

    // Setup RPC client
    let transport = EioTransport::<_, 512>::new(uart);
    let mut client = Client::<_, Request, Response>::new(transport, Duration::from_millis(100));

    let strings = hoshiguma_core::telemetry_module::build_string_registry().unwrap();

    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    'connection: loop {
        // Report telemetry inoperative
        status_tx
            .publish((MonitorKind::TelemetryInop, Severity::Information))
            .await;

        // Wait for telemetry module to come online
        wait_for_telemetry_module_ready(&mut client).await;

        // Send static strings to telemetry module
        if let Err(e) = populate_telemetry_module_string_registry(&mut client, &strings).await {
            error!("Failed to send strings to telemetry module: {}", e);
            Timer::after_secs(1).await;
            continue 'connection;
        }

        // Get time from telemetry module
        let time = match get_time_offset_from_telemetry_module(&mut client).await {
            Ok(time) => {
                info!("Got time from telemetry module: {}", time);
                NtpSyncedTime::new(
                    TimeDelta::microseconds(Instant::now().as_micros() as i64),
                    time,
                )
            }
            Err(e) => {
                error!("Failed to get time from telemetry module: {}", e);
                Timer::after_secs(1).await;
                continue 'connection;
            }
        };

        // Report telemetry ready
        status_tx
            .publish((MonitorKind::TelemetryInop, Severity::Normal))
            .await;

        // Receive data points from queue
        loop {
            let data_point = TELEMETRY_TX.receive().await;

            match data_point.to_templated_data_point(&strings) {
                Ok(mut data_point) => {
                    // Set the actual time for the data point
                    data_point.timestamp =
                        Some(time.now(TimeDelta::microseconds(Instant::now().as_micros() as i64)));

                    debug!("Submitting data point {}", data_point);
                    match client
                        .call(
                            Request::SendTelemetryDataPoint(data_point),
                            Duration::from_millis(50),
                        )
                        .await
                    {
                        Ok(_) => {
                            debug!("Data point submitted successfully");
                        }
                        Err(e) => {
                            warn!("RPC call failed when submitting data point: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Failed to template data point: {}", e);
                    DATA_POINT_TEMPLATE_ERRORS.fetch_add(1, Ordering::Relaxed);
                }
            }
        }
    }
}

static TELEMETRY_TX: Channel<CriticalSectionRawMutex, StaticTelemetryDataPoint, 32> =
    Channel::new();

pub(crate) fn queue_telemetry_data_point(data_point: StaticTelemetryDataPoint) {
    if let Err(e) = TELEMETRY_TX.try_send(data_point) {
        warn!("Data point discarded: {}", e);
        DATA_POINTS_DISCARDED.fetch_add(1, Ordering::Relaxed);
    }
}

async fn wait_for_telemetry_module_ready<T: Transport<RpcMessage<Request, Response>>>(
    client: &mut Client<'_, T, Request, Response>,
) {
    loop {
        match is_telemetry_module_ready(client).await {
            Ok(true) => return,
            Ok(false) => (),
            Err(_) => (),
        }

        Timer::after_secs(1).await;
    }
}

async fn is_telemetry_module_ready<T: Transport<RpcMessage<Request, Response>>>(
    client: &mut Client<'_, T, Request, Response>,
) -> Result<bool, teeny_rpc::Error> {
    match client
        .call(Request::IsReady, Duration::from_millis(500))
        .await
    {
        Ok(Response::IsReady(ready)) => {
            info!("Telemetry module ready: {}", ready);
            Ok(ready)
        }
        Ok(_) => {
            warn!("Telemetry module questionably ready: incorrect response message type");
            Err(teeny_rpc::Error::IncorrectMessageType)
        }
        Err(teeny_rpc::Error::Timeout) => {
            warn!("Telemetry module not ready: timeout");
            Ok(false)
        }
        Err(e) => {
            warn!("Telemetry module questionably ready: error {}", e);
            Err(e)
        }
    }
}

async fn populate_telemetry_module_string_registry<T: Transport<RpcMessage<Request, Response>>>(
    client: &mut Client<'_, T, Request, Response>,
    strings: &StringRegistry,
) -> Result<(), ()> {
    let timeout = Duration::from_millis(500);

    // Clear existing strings
    debug!("Clearing existing strings");
    match client.call(Request::ClearStringRegistry, timeout).await {
        Ok(Response::ClearStringRegistry) => Ok(()),
        Ok(_) => Err(()),
        Err(_) => Err(()),
    }?;

    // Send new strings
    debug!("Sending new strings");
    for s in strings.iter() {
        debug!("Sending string: {}", s);
        match client
            .call(Request::PushStringToRegistry(s.clone()), timeout)
            .await
        {
            Ok(Response::PushStringToRegistry(Ok(_))) => Ok(()),
            Ok(Response::PushStringToRegistry(Err(e))) => {
                error!("Failed pushing string, module response: {}", e);
                Err(())
            }
            Ok(_) => Err(()),
            Err(_) => Err(()),
        }?;
    }

    // Check metadata matches
    debug!("Checking string metadata matches");
    let metadata = match client
        .call(Request::GetStringRegistryMetadata, timeout)
        .await
    {
        Ok(Response::GetStringRegistryMetadata(metadata)) => Ok(metadata),
        Ok(_) => Err(()),
        Err(_) => Err(()),
    }?;
    debug!("Got from module: {}", metadata);

    if strings.metadata() == metadata {
        Ok(())
    } else {
        error!(
            "String metadata mismatch: (local) {} != (remote) {}",
            strings.metadata(),
            metadata,
        );
        Err(())
    }
}

async fn get_time_offset_from_telemetry_module<T: Transport<RpcMessage<Request, Response>>>(
    client: &mut Client<'_, T, Request, Response>,
) -> Result<DateTime<Utc>, ()> {
    match client
        .call(Request::GetWallTime, Duration::from_millis(500))
        .await
    {
        Ok(Response::GetWallTime(Ok(time))) => {
            info!("Telemetry module ready");
            Ok(time)
        }
        Ok(Response::GetWallTime(Err(_))) => {
            warn!("Get time error: telemetry module fail");
            Err(())
        }
        Ok(_) => {
            warn!("Get time error: incorrect response message type");
            Err(())
        }
        Err(e) => {
            warn!("Get time error {}", e);
            Err(())
        }
    }
}
