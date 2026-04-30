use crate::telemetry_tx::TELEMETRY_TX;
use defmt::warn;
use embassy_net::Stack;
use hoshiguma_api::{
    Message,
    telemetry_bridge::{Request, Response, ResponseData},
};
use hoshiguma_common::network::message_handler_loop;

pub(crate) const NUM_LISTENERS: usize = 2;

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
pub(super) async fn task(stack: Stack<'static>, stack_external: Stack<'static>, id: usize) {
    let telem_pub = TELEMETRY_TX.publisher().unwrap();

    message_handler_loop(stack, id, async |mut message| {
        let request = match message.payload::<Request>() {
            Ok(request) => request,
            Err(_) => {
                warn!("socket {}: failed to parse request", id);
                return Message::new(&Response(Err(()))).unwrap();
            }
        };

        let response = match request {
            Request::IsReady => Response(Ok(ResponseData::Ready(
                stack_external.is_link_up() && stack_external.is_config_up(),
            ))),
            Request::GetTime => Response(Ok(ResponseData::Time(crate::wall_time::now()))),
            Request::SendTelemetryDataPoint(data_point) => {
                telem_pub.publish(data_point).await;
                Response(Ok(ResponseData::TelementryDataPointAck))
            }
        };

        match Message::new(&response) {
            Ok(message) => message,
            Err(_) => {
                warn!("socket {}: failed to serialize response", id);
                Message::new(&Response(Err(()))).unwrap()
            }
        }
    })
    .await
}
