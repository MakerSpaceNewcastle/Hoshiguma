use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_message,
    CoolantResevoirLevelSensorResources,
};
use defmt::Format;
use embassy_futures::select::select;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};

#[derive(Clone, Format)]
pub(crate) struct CoolantResevoirLevelReading(pub(crate) Result<CoolantResevoirLevel, ()>);

impl From<&CoolantResevoirLevelReading>
    for hoshiguma_telemetry_protocol::payload::observation::CoolantResevoirLevelReading
{
    fn from(value: &CoolantResevoirLevelReading) -> Self {
        match value.0 {
            Ok(CoolantResevoirLevel::Full) => {
                Ok(hoshiguma_telemetry_protocol::payload::observation::CoolantResevoirLevel::Full)
            }
            Ok(CoolantResevoirLevel::Low) => {
                Ok(hoshiguma_telemetry_protocol::payload::observation::CoolantResevoirLevel::Low)
            }
            Ok(CoolantResevoirLevel::Empty) => {
                Ok(hoshiguma_telemetry_protocol::payload::observation::CoolantResevoirLevel::Empty)
            }
            Err(_) => Err(()),
        }
    }
}

#[derive(Clone, Format)]
pub(crate) enum CoolantResevoirLevel {
    Full,
    Low,
    Empty,
}

pub(crate) static COOLANT_RESEVOIR_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    CoolantResevoirLevelReading,
    2,
> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: CoolantResevoirLevelSensorResources) {
    let empty = Input::new(r.empty, Pull::Down);
    let low = Input::new(r.low, Pull::Down);

    let mut empty = PolledInput::new(empty, Duration::from_millis(200));
    let mut low = PolledInput::new(low, Duration::from_millis(500));

    let tx = COOLANT_RESEVOIR_LEVEL_CHANGED.sender();

    loop {
        select(empty.wait_for_change(), low.wait_for_change()).await;

        let state = CoolantResevoirLevelReading(match (empty.level().await, low.level().await) {
            (Level::Low, Level::Low) => Ok(CoolantResevoirLevel::Full),
            (Level::Low, Level::High) => Err(()),
            (Level::High, Level::Low) => Ok(CoolantResevoirLevel::Low),
            (Level::High, Level::High) => Ok(CoolantResevoirLevel::Empty),
        });

        queue_telemetry_message(Payload::Observation(
            ObservationPayload::CoolantResevoirLevel((&state).into()),
        ))
        .await;

        tx.send(state);
    }
}
