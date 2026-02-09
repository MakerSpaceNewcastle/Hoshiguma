use crate::telemetry_tx::TELEMETRY_TX;
use defmt::warn;
use embassy_net::Stack;
use hoshiguma_api::{
    Message,
    telemetry_bridge::{request, response},
};
use hoshiguma_common::network::message_handler_loop;

pub(crate) const NUM_LISTENERS: usize = 2;

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
pub(super) async fn task(stack: Stack<'static>, stack_external: Stack<'static>, id: usize) {
    let telem_pub = TELEMETRY_TX.publisher().unwrap();

    message_handler_loop(stack, id, async |mut message| {
        let response = if message.payload::<request::IsReady>().is_ok() {
            Message::new(&response::Ready(
                stack_external.is_link_up() && stack_external.is_config_up(),
            ))
            .ok()
        } else if message.payload::<request::GetTime>().is_ok() {
            Message::new(&response::Time(crate::wall_time::now())).ok()
        } else if let Ok(data_point) = message.payload::<request::SendTelemetryDataPoint>() {
            telem_pub.publish(data_point.0).await;
            Message::new(&response::TelemetryDataPointAck).ok()
        } else {
            None
        };

        match response {
            Some(response) => response,
            None => {
                warn!("API error, no response created");
                // Return an API error if no response was generated
                Message::new(&response::ApiError).expect("API error failed to serialise")
            }
        }
    })
    .await
}
