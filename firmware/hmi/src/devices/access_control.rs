use crate::{AccessControlResources, Notification};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Sender};
use embassy_time::Timer;
use hoshiguma_api::{AccessControlRawInput, AccessControlState};
use peek_o_display_bsp::embassy_rp::gpio::{Input, Level, Pull};

#[embassy_executor::task]
pub(crate) async fn task(
    r: AccessControlResources,
    notif_tx: Sender<'static, CriticalSectionRawMutex, Notification, 8>,
) {
    let granted_signal = Input::new(r.granted, Pull::None);
    let denied_signal = Input::new(r.denied, Pull::None);

    // TODO
    loop {
        let granted = granted_signal.get_level();
        let denied = denied_signal.get_level();

        let raw_state = match (granted, denied) {
            (Level::High, Level::Low) => AccessControlRawInput::Granted,
            (Level::Low, Level::High) => AccessControlRawInput::Denied,
            (Level::Low, Level::Low) => AccessControlRawInput::Idle,
            (Level::High, Level::High) => AccessControlRawInput::Idle,
        };

        let state: AccessControlState = raw_state.clone().into();

        notif_tx
            .send(Notification::AccessControlInputChanged(raw_state))
            .await;
        notif_tx
            .send(Notification::AccessControlStateChanged(state))
            .await;

        Timer::after_millis(250).await;
    }
}
