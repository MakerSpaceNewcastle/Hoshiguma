use crate::{
    self_telemetry::{DATA_POINTS_DISCARDED_FORMAT_FAIL, DATA_POINTS_DISCARDED_QUEUE_FULL},
    telemetry_bridge_comm::wait_for_telemetry_bridge_ready,
};
use core::sync::atomic::Ordering;
use defmt::{debug, error, info, warn};
use embassy_net::{
    Stack,
    tcp::{State, TcpSocket},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Duration;
use hoshiguma_api::{
    CONTROL_PORT, TELEMETRY_BRIDGE_IP_ADDRESS,
    telemetry_bridge::{
        FormattedTelemetryDataPoint, Request, Response, ResponseData, TELEMETRY_DATA_POINT_MAX_LEN,
    },
};
use hoshiguma_common::{
    network::{send_request_socket, try_connect},
    telemetry::FormatInfluxResult,
};

static TELEMETRY_TX: Channel<CriticalSectionRawMutex, FormattedTelemetryDataPoint, 64> =
    Channel::new();

pub(crate) fn queue_telemetry_data_point(
    data_point: FormatInfluxResult<TELEMETRY_DATA_POINT_MAX_LEN>,
) {
    if let Ok(data_point) = data_point {
        if let Err(e) = TELEMETRY_TX.try_send(FormattedTelemetryDataPoint(data_point)) {
            warn!("Data point discarded: {}", e);
            DATA_POINTS_DISCARDED_QUEUE_FULL.fetch_add(1, Ordering::Relaxed);
        }
    } else {
        warn!("Data point discarded: failed to format data point");
        DATA_POINTS_DISCARDED_FORMAT_FAIL.fetch_add(1, Ordering::Relaxed);
    }
}

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("telemetry").await;

    let telem_rx = TELEMETRY_TX.receiver();

    // TODO
    // let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    'connection: loop {
        // Report telemetry inoperative
        // TODO
        // status_tx
        //     .publish((MonitorKind::TelemetryInop, Severity::Information))
        //     .await;

        // Wait for telemetry bridge to come online
        wait_for_telemetry_bridge_ready(stack).await;

        // Report telemetry ready
        // TODO
        // status_tx
        //     .publish((MonitorKind::TelemetryInop, Severity::Normal))
        //     .await;

        let mut rx_buffer = [0; 4096];
        let mut tx_buffer = [0; 4096];

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_keep_alive(Some(Duration::from_millis(100)));
        socket.set_timeout(Some(Duration::from_secs(1)));

        // Receive data points from queue
        loop {
            let data_point = telem_rx.receive().await;

            if socket.state() == State::Closed {
                if let Err(e) =
                    try_connect(&mut socket, TELEMETRY_BRIDGE_IP_ADDRESS, CONTROL_PORT).await
                {
                    error!("Failed to connect to telemetry bridge: {}", e);
                    continue 'connection;
                }
            }

            info!("Sending data point: {}", data_point.0);
            match send_request_socket::<_, Response>(
                &mut socket,
                &Request::SendTelemetryDataPoint(data_point),
            )
            .await
            {
                Ok(response) => match response.0 {
                    Ok(ResponseData::TelementryDataPointAck) => {
                        debug!("Data point ack");
                    }
                    response => {
                        warn!("Unexpected response: {}", response);
                    }
                },
                Err(e) => {
                    warn!("Failed to send request: {}", e);
                    continue 'connection;
                }
            }

            if telem_rx.is_empty() {
                socket.close();
            }
        }
    }
}
