use chrono::{DateTime, Utc};
use defmt::{info, warn};
use embassy_net::Stack;
use embassy_time::Timer;
use hoshiguma_api::{
    CONTROL_PORT, TELEMETRY_BRIDGE_IP_ADDRESS,
    telemetry_bridge::{Request, Response, ResponseData},
};
use hoshiguma_common::network::send_request;

pub(crate) async fn wait_for_telemetry_bridge_ready(stack: Stack<'static>) {
    loop {
        if is_telemetry_bridge_ready(stack).await {
            return;
        }

        Timer::after_secs(1).await;
    }
}

pub(crate) async fn is_telemetry_bridge_ready(stack: Stack<'static>) -> bool {
    match send_request::<_, Response>(
        stack,
        TELEMETRY_BRIDGE_IP_ADDRESS,
        CONTROL_PORT,
        &Request::IsReady,
    )
    .await
    {
        Ok(response) => match response.0 {
            Ok(ResponseData::Ready(ready)) => {
                info!("Telemetry module ready: {}", ready);
                ready
            }
            response => {
                warn!("Unexpected response: {}", response);
                false
            }
        },
        Err(e) => {
            warn!("Failed to send request: {}", e);
            false
        }
    }
}

pub(crate) async fn get_time_from_telemetry_bridge(
    stack: Stack<'static>,
) -> Result<DateTime<Utc>, ()> {
    match send_request::<_, Response>(
        stack,
        TELEMETRY_BRIDGE_IP_ADDRESS,
        CONTROL_PORT,
        &Request::GetTime,
    )
    .await
    {
        Ok(response) => match response.0 {
            Ok(ResponseData::Time(Some(time))) => Ok(time),
            Ok(ResponseData::Time(None)) => {
                warn!("Telemetry bridge does not have a valid time");
                Err(())
            }
            response => {
                warn!("Unexpected response: {}", response);
                Err(())
            }
        },
        Err(e) => {
            warn!("Failed to send request: {}", e);
            Err(())
        }
    }
}
