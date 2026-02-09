use crate::{
    api::access_control_raw_input_rx,
    logic::{hmi_status_screen::hmi_status_screen_info_rx, machine_power::machine_power_rx},
};
use defmt::info;
use embassy_futures::select::{Either4, select4};
use embassy_net::Stack;
use embassy_time::{Duration, Ticker};
use hoshiguma_api::{
    DesiredMachinePower, HMI_IP_ADDRESS,
    hmi::{AccessControlRawInput, BacklightMode, Screen},
};
use hoshiguma_common::{
    changed::Changed, network::send_request, remote_state_reconciler::RemoteStateReconciler,
};

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("hmi").await;

    let mut access_control_raw_input_rx = access_control_raw_input_rx();
    let mut desired_machine_power_rx = machine_power_rx();
    let mut status_screen_info_rx = hmi_status_screen_info_rx();

    let mut hmi_backlight = RemoteStateReconciler::<
        hoshiguma_api::hmi::to_hmi::request::SetBacklight,
        _,
    >::new(stack, HMI_IP_ADDRESS);

    let mut reconcile_tick = Ticker::every(Duration::from_secs(10));

    loop {
        match select4(
            reconcile_tick.next(),
            access_control_raw_input_rx.changed(),
            desired_machine_power_rx.changed(),
            status_screen_info_rx.changed(),
        )
        .await
        {
            Either4::First(_) => {
                let _ = hmi_backlight.reconcile().await;
            }
            Either4::Second(state) => {
                // Wake the backlight if the denied signal is given
                if state == AccessControlRawInput::Denied
                    && send_request(
                        stack,
                        HMI_IP_ADDRESS,
                        2000,
                        3,
                        &hoshiguma_api::hmi::to_hmi::request::BacklightWake,
                    )
                    .await
                    .is_err()
                {
                    info!("Failed to send request to HMI");
                }

                // In any case, switch to the status screen to show the access control indicator
                if send_request(
                    stack,
                    HMI_IP_ADDRESS,
                    2000,
                    3,
                    &hoshiguma_api::hmi::to_hmi::request::ShowScreen(Screen::Status),
                )
                .await
                .is_err()
                {
                    info!("Failed to send request to HMI");
                }
            }
            Either4::Third(state) => {
                let state = match state {
                    DesiredMachinePower::Off => BacklightMode::Auto,
                    DesiredMachinePower::On => BacklightMode::AlwaysOn,
                };

                if hmi_backlight
                    .set_desired_state(hoshiguma_api::hmi::to_hmi::request::SetBacklight(state))
                    == Changed::Yes
                {
                    info!("Triggering reconciliation now");
                    reconcile_tick.reset();
                }
            }
            Either4::Fourth(info) => {
                if send_request(
                    stack,
                    HMI_IP_ADDRESS,
                    2000,
                    3,
                    &hoshiguma_api::hmi::to_hmi::request::SetStatusScreenInfo(info),
                )
                .await
                .is_err()
                {
                    info!("Failed to send request to HMI");
                }
            }
        }
    }
}
