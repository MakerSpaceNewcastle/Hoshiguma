use crate::AccessControlResources;
use embassy_time::Timer;
use hoshiguma_api::{AccessControlRawInput, AccessControlState};
use peek_o_display_bsp::embassy_rp::gpio::{Input, Level, Pull};

#[embassy_executor::task]
pub(crate) async fn task(r: AccessControlResources) {
    let granted_signal = Input::new(r.granted, Pull::None);
    let denied_signal = Input::new(r.denied, Pull::None);

    loop {
        let granted = granted_signal.get_level();
        let denied = denied_signal.get_level();

        let raw_state = match (granted, denied) {
            (Level::High, Level::Low) => AccessControlRawInput::Granted,
            (Level::Low, Level::High) => AccessControlRawInput::Denied,
            (Level::Low, Level::Low) => AccessControlRawInput::Idle,
            (Level::High, Level::High) => AccessControlRawInput::Idle,
        };

        let state: AccessControlState = raw_state.into();

        // TODO

        Timer::after_millis(250).await;
    }
}
