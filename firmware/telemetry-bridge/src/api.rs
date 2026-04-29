use defmt::warn;
use embassy_net::Stack;
use hoshiguma_api::{
    Message,
    telemetry_bridge::{Request, Response, ResponseData},
};
use hoshiguma_common::network::message_handler_loop;

pub(crate) const NUM_LISTENERS: usize = 2;

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
pub(super) async fn listen_task(stack: Stack<'static>, id: usize) {
    message_handler_loop(stack, id, async |mut message| {
        let request = match message.payload::<Request>() {
            Ok(request) => request,
            Err(_) => {
                warn!("socket {}: failed to parse request", id);
                return Message::new(&Response(Err(()))).unwrap();
            }
        };

        let response = match request {
            Request::IsReady => {
                // TODO
                Response(Ok(ResponseData::Ready(false)))
            }
            Request::GetTime => Response(Ok(ResponseData::Time(crate::wall_time::now()))),
            Request::SendTelemetryDataPoint(data_point) => {
                todo!()
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
