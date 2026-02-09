//! Interface for the access control system, using the granted and denied LEDs as signals.

use crate::{AccessControlResources, Notification, api::NOTIFICATIONS};
use defmt::debug;
use embassy_futures::select::{Either3, select3};
use embassy_time::Timer;
use hoshiguma_api::hmi::{AccessControlRawInput, AccessControlState};
use peek_o_display_bsp::embassy_rp::gpio::{Input, Level, Pull};

#[embassy_executor::task]
pub(crate) async fn task(r: AccessControlResources) {
    let mut granted_signal = Input::new(r.granted, Pull::None);
    let mut denied_signal = Input::new(r.denied, Pull::None);

    loop {
        match select3(
            Timer::after_secs(5),
            granted_signal.wait_for_any_edge(),
            denied_signal.wait_for_any_edge(),
        )
        .await
        {
            Either3::First(_) => {
                debug!("checking: interval");
            }
            Either3::Second(_) => {
                debug!("checking: granted signal");
            }
            Either3::Third(_) => {
                debug!("checking: denied signal");
            }
        }

        // Little delay to allow any further signal changes
        Timer::after_millis(10).await;

        let granted = granted_signal.get_level();
        let denied = denied_signal.get_level();

        let raw_state = match (granted, denied) {
            (Level::Low, Level::High) => AccessControlRawInput::Granted,
            (Level::High, Level::Low) => AccessControlRawInput::Denied,
            (Level::High, Level::High) => AccessControlRawInput::Idle,
            (Level::Low, Level::Low) => AccessControlRawInput::Idle,
        };

        let state: AccessControlState = raw_state.into();

        NOTIFICATIONS
            .send(Notification::AccessControlInputChanged(raw_state))
            .await;
        NOTIFICATIONS
            .send(Notification::AccessControlStateChanged(state))
            .await;
    }
}
