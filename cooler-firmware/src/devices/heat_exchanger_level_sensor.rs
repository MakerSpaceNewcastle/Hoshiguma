use crate::{
    polled_input::PolledInput,
    HeatExchangerLevelSensorResources,
};
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_protocol::cooler::types::HeatExchangeFluidLevel;

pub(crate) static HEAT_EXCHANGER_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    HeatExchangeFluidLevel,
    2,
> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: HeatExchangerLevelSensorResources) {
    let low = Input::new(r.low, Pull::Down);
    let mut low = PolledInput::new(low, Duration::from_millis(500));

    let tx = HEAT_EXCHANGER_LEVEL_CHANGED.sender();

    loop {
        let new = low.wait_for_change().await;

        let state = match (empty.level().await, low.level().await) {
            (Level::Low, Level::Low) => Ok(CoolantResevoirLevel::Full),
            (Level::Low, Level::High) => Err(()),
            (Level::High, Level::Low) => Ok(CoolantResevoirLevel::Low),
            (Level::High, Level::High) => Ok(CoolantResevoirLevel::Empty),
        };

        queue_telemetry_event(EventKind::Observation(
            ObservationEvent::CoolantResevoirLevel(state.clone()),
        ))
        .await;

        tx.send(state);
    }
}
