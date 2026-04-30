use crate::self_telemetry::{DATA_POINTS_DISCARDED_BUFFER, DATA_POINTS_DISCARDED_FORMAT};
use chrono::{DateTime, Utc};
use core::sync::atomic::Ordering;
use defmt::warn;
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use hoshiguma_api::telemetry_bridge::{FormattedTelemetryDataPoint, TELEMETRY_DATA_POINT_MAX_LEN};
use hoshiguma_common::telemetry::FormatInfluxResult;

static TELEMETRY_TX: Channel<CriticalSectionRawMutex, FormattedTelemetryDataPoint, 64> =
    Channel::new();

pub(crate) fn queue_telemetry_data_point(
    data_point: FormatInfluxResult<TELEMETRY_DATA_POINT_MAX_LEN>,
) {
    if let Ok(data_point) = data_point {
        if let Err(e) = TELEMETRY_TX.try_send(FormattedTelemetryDataPoint(data_point)) {
            warn!("Data point discarded: {}", e);
            DATA_POINTS_DISCARDED_BUFFER.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        warn!("Data point discarded: failed to format data point");
        DATA_POINTS_DISCARDED_FORMAT.fetch_add(1, Ordering::Relaxed);
    }
}

// #[embassy_executor::task]
// pub(super) async fn task(r: TelemetryResources) {
//     #[cfg(feature = "trace")]
//     crate::trace::name_task("telemetry").await;

//     const TX_BUFFER_SIZE: usize = 256;
//     static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
//     let tx_buffer = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

//     const RX_BUFFER_SIZE: usize = 32;
//     static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
//     let rx_buffer = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

//     let mut config = UartConfig::default();
//     config.baudrate = hoshiguma_core::telemetry_module::SERIAL_BAUD;

//     let uart = BufferedUart::new(
//         r.uart, r.tx_pin, r.rx_pin, Irqs, tx_buffer, rx_buffer, config,
//     );

//     // Setup RPC client
//     let transport = EioTransport::<_, 512>::new(uart);
//     let mut client = Client::<_, Request, Response>::new(transport, Duration::from_millis(100));

//     let strings = hoshiguma_core::telemetry_module::build_string_registry().unwrap();

//     let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

//     'connection: loop {
//         // Report telemetry inoperative
//         status_tx
//             .publish((MonitorKind::TelemetryInop, Severity::Information))
//             .await;

//         // Wait for telemetry module to come online
//         wait_for_telemetry_module_ready(&mut client).await;

//         // Send static strings to telemetry module
//         if let Err(e) = populate_telemetry_module_string_registry(&mut client, &strings).await {
//             error!("Failed to send strings to telemetry module: {}", e);
//             Timer::after_secs(1).await;
//             continue 'connection;
//         }

//         // Get time from telemetry module
//         let time = match get_time_offset_from_telemetry_module(&mut client).await {
//             Ok(time) => {
//                 info!("Got time from telemetry module: {}", time);
//                 NtpSyncedTime::new(
//                     TimeDelta::microseconds(Instant::now().as_micros() as i64),
//                     time,
//                 )
//             }
//             Err(e) => {
//                 error!("Failed to get time from telemetry module: {}", e);
//                 Timer::after_secs(1).await;
//                 continue 'connection;
//             }
//         };

//         // Report telemetry ready
//         status_tx
//             .publish((MonitorKind::TelemetryInop, Severity::Normal))
//             .await;

//         // Receive data points from queue
//         loop {
//             let data_point = TELEMETRY_TX.receive().await;

//             match data_point.to_templated_data_point(&strings) {
//                 Ok(mut data_point) => {
//                     // Set the actual time for the data point
//                     data_point.timestamp =
//                         Some(time.now(TimeDelta::microseconds(Instant::now().as_micros() as i64)));

//                     debug!("Submitting data point {}", data_point);
//                     match client
//                         .call(
//                             Request::SendTelemetryDataPoint(data_point),
//                             Duration::from_millis(50),
//                         )
//                         .await
//                     {
//                         Ok(_) => {
//                             debug!("Data point submitted successfully");
//                         }
//                         Err(e) => {
//                             warn!("RPC call failed when submitting data point: {}", e);
//                         }
//                     }
//                 }
//                 Err(e) => {
//                     warn!("Failed to template data point: {}", e);
//                     DATA_POINT_TEMPLATE_ERRORS.fetch_add(1, Ordering::Relaxed);
//                 }
//             }
//         }
//     }
// }

async fn wait_for_telemetry_module_ready(stack: Stack<'static>) {
    loop {
        match is_telemetry_module_ready(stack).await {
            Ok(true) => return,
            Ok(false) => (),
            Err(_) => (),
        }

        Timer::after_secs(1).await;
    }
}

async fn is_telemetry_module_ready(stack: Stack<'static>) -> Result<bool, ()> {
    todo!()
}

async fn get_time_from_telemetry_module(stack: Stack<'static>) -> Result<DateTime<Utc>, ()> {
    todo!()
}
