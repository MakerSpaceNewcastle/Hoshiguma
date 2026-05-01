use chrono::{DateTime, Utc};
use defmt::warn;
use embassy_net::Stack;
use hoshiguma_api::{
    CONTROL_PORT, TELEMETRY_BRIDGE_IP_ADDRESS,
    telemetry_bridge::{Request, Response, ResponseData},
};
use hoshiguma_common::network::send_request;

async fn get_time_from_telemetry_bridge(stack: Stack<'static>) -> Result<DateTime<Utc>, ()> {
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
