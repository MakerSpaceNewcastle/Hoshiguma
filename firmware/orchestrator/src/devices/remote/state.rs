use crate::{
    logic::{
        cooling::{compressor_rx, coolant_pump_rx, radiator_fan_rx},
        status_light::status_light_rx,
    },
    telemetry::queue_telemetry_data_point,
};
use defmt::{debug, info};
use embassy_futures::select::{Either5, select5};
use embassy_net::Stack;
use embassy_time::{Duration, Instant, Ticker};
use hoshiguma_api::{COOLER_IP_ADDRESS, REAR_SENSOR_BOARD_IP_ADDRESS};
use hoshiguma_common::{
    changed::Changed, remote_state_reconciler::RemoteStateReconciler, telemetry::format_influx_line,
};

#[embassy_executor::task]
pub(crate) async fn task(stack: Stack<'static>) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("remote device state").await;

    let mut cooler_pump = RemoteStateReconciler::<
        hoshiguma_api::cooler::request::SetCoolantPumpState,
        _,
    >::new(stack, COOLER_IP_ADDRESS);

    let mut cooler_radiator_fan = RemoteStateReconciler::<
        hoshiguma_api::cooler::request::SetRadiatorFanState,
        _,
    >::new(stack, COOLER_IP_ADDRESS);

    let mut cooler_compressor = RemoteStateReconciler::<
        hoshiguma_api::cooler::request::SetCompressorState,
        _,
    >::new(stack, COOLER_IP_ADDRESS);

    let mut status_light = RemoteStateReconciler::<
        hoshiguma_api::rear_sensor_board::request::SetStatusLight,
        _,
    >::new(stack, REAR_SENSOR_BOARD_IP_ADDRESS);

    let tick_interval = Duration::from_secs(2);
    let mut tick = Ticker::every(tick_interval);

    let mut coolant_pump_rx = coolant_pump_rx();
    let mut radiator_fan_rx = radiator_fan_rx();
    let mut compressor_rx = compressor_rx();
    let mut status_light_rx = status_light_rx();

    loop {
        match select5(
            tick.next(),
            coolant_pump_rx.changed(),
            radiator_fan_rx.changed(),
            compressor_rx.changed(),
            status_light_rx.changed(),
        )
        .await
        {
            Either5::First(_) => {
                debug!("Reconciling remote states");

                if let Ok(Some(response)) = cooler_pump.reconcile().await
                    && let Ok(state) = response.0
                {
                    queue_telemetry_data_point(format_influx_line(
                        format_args!("coolant_pump value=\"{state}\""),
                        crate::wall_time::now(),
                    ));
                }

                if let Ok(Some(response)) = cooler_radiator_fan.reconcile().await
                    && let Ok(state) = response.0
                {
                    queue_telemetry_data_point(format_influx_line(
                        format_args!("radiator_fan value=\"{state}\""),
                        crate::wall_time::now(),
                    ));
                }

                if let Ok(Some(response)) = cooler_compressor.reconcile().await
                    && let Ok(state) = response.0
                {
                    queue_telemetry_data_point(format_influx_line(
                        format_args!("compressor value=\"{state}\""),
                        crate::wall_time::now(),
                    ));
                }

                let _ = status_light.reconcile().await;
                // We don't care about telemetry for the status light since it is:
                // a) derived from other states anyway, and
                // b) not trivial to report given the time based pattern representation
                // But if it was ever desired to report it, this is where it would be done.
            }
            Either5::Second(state) => {
                if cooler_pump
                    .set_desired_state(hoshiguma_api::cooler::request::SetCoolantPumpState(state))
                    == Changed::Yes
                {
                    info!("Triggering reconciliation now");
                    tick.reset_at(Instant::now().saturating_sub(tick_interval));
                }
            }
            Either5::Third(state) => {
                if cooler_radiator_fan
                    .set_desired_state(hoshiguma_api::cooler::request::SetRadiatorFanState(state))
                    == Changed::Yes
                {
                    info!("Triggering reconciliation now");
                    tick.reset_at(Instant::now().saturating_sub(tick_interval));
                }
            }
            Either5::Fourth(state) => {
                if cooler_compressor
                    .set_desired_state(hoshiguma_api::cooler::request::SetCompressorState(state))
                    == Changed::Yes
                {
                    info!("Triggering reconciliation now");
                    tick.reset_at(Instant::now().saturating_sub(tick_interval));
                }
            }
            Either5::Fifth(state) => {
                if status_light.set_desired_state(
                    hoshiguma_api::rear_sensor_board::request::SetStatusLight(state),
                ) == Changed::Yes
                {
                    info!("Triggering reconciliation now");
                    tick.reset_at(Instant::now().saturating_sub(tick_interval));
                }
            }
        }
    }
}
