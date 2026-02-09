use chrono::{DateTime, Utc};
use defmt::{info, warn};
use embassy_net::Stack;
use embassy_time::Timer;
use hoshiguma_api::{
    API_PORT, TELEMETRY_BRIDGE_IP_ADDRESS,
    telemetry_bridge::{request, response},
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
    match send_request::<_, response::Ready>(
        stack,
        TELEMETRY_BRIDGE_IP_ADDRESS,
        API_PORT,
        20,
        &request::IsReady,
    )
    .await
    {
        Ok(response) => {
            info!("Telemetry module ready: {}", response.0);
            response.0
        }
        Err(e) => {
            warn!("Failed to send request: {}", e);
            false
        }
    }
}

pub(crate) async fn get_time_from_telemetry_bridge(
    stack: Stack<'static>,
) -> Result<DateTime<Utc>, ()> {
    match send_request::<_, response::Time>(
        stack,
        TELEMETRY_BRIDGE_IP_ADDRESS,
        API_PORT,
        5,
        &request::GetTime,
    )
    .await
    {
        Ok(response) => match response.0 {
            Some(time) => Ok(time),
            None => {
                warn!("Telemetry bridge does not have a valid time");
                Err(())
            }
        },
        Err(e) => {
            warn!("Failed to send request: {}", e);
            Err(())
        }
    }
}
