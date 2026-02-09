use crate::telemetry::queue_telemetry_data_point;
use defmt::warn;
use embassy_net::Stack;
use hoshiguma_api::{
    Message,
    hmi::{AccessControlRawInput, AccessControlState, from_hmi},
};
use hoshiguma_common::{network::message_handler_loop, telemetry::format_influx_line};

crate::variable_watch!(access_control_raw_input, AccessControlRawInput, 2);
crate::variable_watch!(access_control_state, AccessControlState, 1);

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("api").await;

    message_handler_loop(stack, 0, async |mut message| {
        let response = if let Ok(state) =
            message.payload::<from_hmi::request::NotifyAccessControlInputChanged>()
        {
            ACCESS_CONTROL_RAW_INPUT.sender().send(state.0);

            queue_telemetry_data_point(format_influx_line(
                format_args!("access_control_raw_input value=\"{}\"", state.0),
                crate::wall_time::now(),
            ));

            Message::new(&from_hmi::response::AckAccessControlInputChanged(state.0)).ok()
        } else if let Ok(state) =
            message.payload::<from_hmi::request::NotifyAccessControlStateChanged>()
        {
            ACCESS_CONTROL_STATE.sender().send(state.0);

            queue_telemetry_data_point(format_influx_line(
                format_args!("access_control_state value=\"{}\"", state.0),
                crate::wall_time::now(),
            ));

            Message::new(&from_hmi::response::AckAccessControlStateChanged(state.0)).ok()
        } else if message
            .payload::<from_hmi::request::NotifyPanelInteraction>()
            .is_ok()
        {
            if let Some(time) = crate::wall_time::now() {
                queue_telemetry_data_point(format_influx_line(
                    format_args!("last_panel_interaction_time value={}", time.timestamp()),
                    crate::wall_time::now(),
                ));
            }

            Message::new(&from_hmi::response::AckPanelInteraction).ok()
        } else {
            None
        };

        match response {
            Some(response) => response,
            None => {
                warn!("API error, no response created");
                // Return an API error if no response was generated
                Message::new(&from_hmi::response::ApiError).expect("API error failed to serialise")
            }
        }
    })
    .await
}
