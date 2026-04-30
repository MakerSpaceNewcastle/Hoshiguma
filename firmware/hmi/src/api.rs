use crate::{DeviceCommunicator, Notification, devices::backlight::BacklightInterfaceChannel};
use defmt::{info, warn};
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Receiver};
use embassy_time::{Duration, Instant};
use hoshiguma_api::{
    Message,
    hmi::to_hmi::{Request, Response, ResponseData},
};
use hoshiguma_common::network::{message_handler_loop, send_request};

pub(crate) const NUM_LISTENERS: usize = 2;
pub(crate) const NUM_NOTIFIERS: usize = 2;

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
pub(super) async fn listen_task(stack: Stack<'static>, id: usize, mut comm: DeviceCommunicator) {
    message_handler_loop(stack, id, async |mut message| {
        let request = match message.payload::<Request>() {
            Ok(request) => request,
            Err(_) => {
                warn!("socket {}: failed to parse request", id);
                return Message::new(&Response(Err(()))).unwrap();
            }
        };

        let _ = crate::COMM_GOOD_INDICATOR.try_send(());

        let response = match request {
            Request::GetGitRevision => Response(Ok(ResponseData::GitRevision(
                git_version::git_version!().try_into().unwrap(),
            ))),
            Request::GetUptime => Response(Ok(ResponseData::Uptime(
                Instant::now().duration_since(Instant::MIN).into(),
            ))),
            Request::GetBootReason => Response(Ok(ResponseData::BootReason(crate::boot_reason()))),
            Request::SetBacklightMode(mode) => Response(
                comm.backlight
                    .set_mode(mode)
                    .await
                    .map(ResponseData::BacklightMode)
                    .map_err(|_| ()),
            ),
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

#[embassy_executor::task(pool_size = NUM_NOTIFIERS)]
pub(super) async fn notify_task(
    stack: Stack<'static>,
    id: usize,
    notif_rx: Receiver<'static, CriticalSectionRawMutex, Notification, 8>,
) {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    loop {
        let notification = notif_rx.receive().await;
        info!("socket {}: got notification: {:?}", id, notification);

        // TODO
        continue;

        let (request, expected_response) = notification.expected_request_and_response();

        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(1)));

        let message = Message::new(&request).unwrap();
        let expected_response = Message::new(&expected_response).unwrap();

        let result = send_request(&mut socket, &message).await;

        let response = match result {
            Ok(response) => response,
            Err(e) => {
                warn!("socket {}: failed to send notification: {}", id, e);
                continue;
            }
        };

        if response != expected_response {
            warn!(
                "socket {}: got unexpected response to notification: {:?} (expected {:?})",
                id, response, expected_response
            );
        }
    }
}
