use crate::{polled_input::PolledInput, rpc::report_event, HeatExchangerLevelSensorResources};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_time::Duration;
use hoshiguma_protocol::cooler::{
    event::{EventKind, ObservationEvent},
    types::HeatExchangeFluidLevel,
};

#[embassy_executor::task]
pub(crate) async fn task(r: HeatExchangerLevelSensorResources) {
    let low = Input::new(r.low, Pull::Down);
    let mut low = PolledInput::new(low, Duration::from_millis(500));

    loop {
        let new = low.wait_for_change().await;

        let state = match new {
            Level::Low => HeatExchangeFluidLevel::Normal,
            Level::High => HeatExchangeFluidLevel::Low,
        };

        report_event(EventKind::Observation(
            ObservationEvent::HeatExchangeFluidLevel(state),
        ))
        .await;
    }
}
