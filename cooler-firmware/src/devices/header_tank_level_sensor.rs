use crate::{polled_input::PolledInput, rpc::report_event, HeaderTankLevelSensorResources};
use embassy_futures::select::select;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_time::Duration;
use hoshiguma_protocol::cooler::{
    event::{EventKind, ObservationEvent},
    types::HeaderTankCoolantLevel,
};

#[embassy_executor::task]
pub(crate) async fn task(r: HeaderTankLevelSensorResources) {
    let empty = Input::new(r.empty, Pull::Down);
    let low = Input::new(r.low, Pull::Down);

    let mut empty = PolledInput::new(empty, Duration::from_millis(200));
    let mut low = PolledInput::new(low, Duration::from_millis(500));

    loop {
        select(empty.wait_for_change(), low.wait_for_change()).await;

        let state = match (empty.level().await, low.level().await) {
            (Level::Low, Level::Low) => Ok(HeaderTankCoolantLevel::Full),
            (Level::Low, Level::High) => Err(()),
            (Level::High, Level::Low) => Ok(HeaderTankCoolantLevel::Normal),
            (Level::High, Level::High) => Ok(HeaderTankCoolantLevel::Empty),
        };

        report_event(EventKind::Observation(
            ObservationEvent::HeaderTankCoolantLevel(state),
        ))
        .await;
    }
}
