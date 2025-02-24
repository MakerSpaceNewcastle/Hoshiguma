use crate::{
    io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    telemetry::queue_telemetry_message,
    CoolantResevoirLevelSensorResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Ticker};
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};

type CoolantResevoirLevelSensor =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 2, CoolantResevoirLevelReading>;

impl From<CoolantResevoirLevelSensorResources> for CoolantResevoirLevelSensor {
    fn from(r: CoolantResevoirLevelSensorResources) -> Self {
        let empty = Input::new(r.empty, Pull::Up);
        let low = Input::new(r.low, Pull::Up);

        Self::new([empty, low])
    }
}

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

impl StateFromDigitalInputs<2> for CoolantResevoirLevelReading {
    fn from_inputs(inputs: [Level; 2]) -> Self {
        Self(match (inputs[0], inputs[1]) {
            (Level::Low, Level::Low) => Ok(CoolantResevoirLevel::Empty),
            (Level::Low, Level::High) => Err(()),
            (Level::High, Level::Low) => Ok(CoolantResevoirLevel::Low),
            (Level::High, Level::High) => Ok(CoolantResevoirLevel::Full),
        })
    }
}

pub(crate) static COOLANT_RESEVOIR_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    CoolantResevoirLevelReading,
    2,
> = Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: CoolantResevoirLevelSensorResources) {
    let mut input: CoolantResevoirLevelSensor = r.into();

    let mut ticker = Ticker::every(Duration::from_millis(500));

    let tx = COOLANT_RESEVOIR_LEVEL_CHANGED.sender();

    loop {
        ticker.next().await;

        if let Some(state) = input.update() {
            queue_telemetry_message(Payload::Observation(
                ObservationPayload::CoolantResevoirLevel((&state).into()),
            ))
            .await;

            tx.send(state);
        }
    }
}
