use crate::{
    Notification,
    ui::{BACKLIGHT_MODE, BACKLIGHT_WAKE, change_screen, set_status_screen_info},
};
use defmt::{debug, error, info, warn};
use embassy_net::Stack;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Instant;
use hoshiguma_api::{
    API_PORT, ExpectedResponse, Message, MessagePayload, ORCHESTRATOR_IP_ADDRESS,
    ResponseVerification, SystemInformation,
    hmi::{
        from_hmi,
        to_hmi::{request, response},
    },
};
use hoshiguma_common::network::{message_handler_loop, send_request};
use serde::{Serialize, de::DeserializeOwned};

pub(crate) const NUM_LISTENERS: usize = 2;
pub(crate) const NUM_NOTIFIERS: usize = 2;

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
pub(super) async fn listen_task(stack: Stack<'static>, id: usize) {
    let backlight_mode_tx = BACKLIGHT_MODE.sender();

    message_handler_loop(stack, id, async |mut message| {
        let response = if message.payload::<request::GetSystemInformation>().is_ok() {
            Message::new(&response::SystemInformation(SystemInformation {
                git_revision: git_version::git_version!().try_into().unwrap(),
                uptime: Instant::now().duration_since(Instant::MIN).into(),
                boot_reason: crate::boot_reason(),
            }))
            .ok()
        } else if let Ok(state) = message.payload::<request::SetBacklight>() {
            backlight_mode_tx.send(state.0);
            Message::new(&response::BacklightMode(Ok(state.0))).ok()
        } else if message.payload::<request::BacklightWake>().is_ok() {
            BACKLIGHT_WAKE.send(()).await;
            Message::new(&response::AckBacklightWake).ok()
        } else if let Ok(state) = message.payload::<request::ShowScreen>() {
            change_screen(state.0).await;
            Message::new(&response::ActiveScreen(state.0)).ok()
        } else if let Ok(state) = message.payload::<request::SetStatusScreenInfo>() {
            set_status_screen_info(state.0);
            Message::new(&response::AckStatusScreenInfo).ok()
        } else {
            None
        };

        match response {
            Some(response) => {
                // Indicate that good communication has happened
                let _ = crate::COMM_GOOD_INDICATOR.try_send(());

                response
            }
            None => {
                warn!("API error, no response created");
                // Return an API error if no response was generated
                Message::new(&response::ApiError).expect("API error failed to serialise")
            }
        }
    })
    .await
}

pub(crate) static NOTIFICATIONS: Channel<CriticalSectionRawMutex, Notification, 8> = Channel::new();

#[embassy_executor::task(pool_size = NUM_NOTIFIERS)]
pub(super) async fn notify_task(stack: Stack<'static>, id: usize) {
    let notif_rx = NOTIFICATIONS.receiver();

    loop {
        let notification = notif_rx.receive().await;
        info!("socket {}: sending notification: {:?}", id, notification);

        match notification {
            Notification::PanelInteraction => {
                send_notification_and_validate(stack, from_hmi::request::NotifyPanelInteraction)
                    .await;
            }
            Notification::AccessControlInputChanged(state) => {
                send_notification_and_validate(
                    stack,
                    from_hmi::request::NotifyAccessControlInputChanged(state),
                )
                .await;
            }
            Notification::AccessControlStateChanged(state) => {
                send_notification_and_validate(
                    stack,
                    from_hmi::request::NotifyAccessControlStateChanged(state),
                )
                .await;
            }
        };
    }
}

async fn send_notification_and_validate<
    ReqT: ResponseVerification<RespT> + ExpectedResponse<Response = RespT> + MessagePayload + Serialize,
    RespT: MessagePayload + DeserializeOwned,
>(
    stack: Stack<'static>,
    request: ReqT,
) {
    // let addr = core::net::Ipv4Addr::new(10, 69, 69, 100);
    let addr = ORCHESTRATOR_IP_ADDRESS;

    match send_request(stack, addr, API_PORT, 3, &request).await {
        Ok(response) => {
            if request.verify_response(&response) {
                debug!("Notification delivered");
            } else {
                error!("Failed to deliver notification: response mismatch");
            }
        }
        Err(e) => {
            error!("Failed to deliver notification: {}", e);
        }
    }
}
