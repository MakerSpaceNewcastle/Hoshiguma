use crate::{
    io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    telemetry::queue_telemetry_message,
    ChassisIntrusionDetectResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Ticker};
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};

type ChassisIntrusionDetector =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 1, ChassisIntrusion>;

impl From<ChassisIntrusionDetectResources> for ChassisIntrusionDetector {
    fn from(r: ChassisIntrusionDetectResources) -> Self {
        let input = Input::new(r.detect, Pull::Down);
        Self::new([input])
    }
}

#[derive(Clone, Format)]
pub(crate) enum ChassisIntrusion {
    Normal,
    Intruded,
}

impl From<&ChassisIntrusion>
    for hoshiguma_telemetry_protocol::payload::observation::ChassisIntrusion
{
    fn from(value: &ChassisIntrusion) -> Self {
        match value {
            ChassisIntrusion::Normal => Self::Normal,
            ChassisIntrusion::Intruded => Self::Intruded,
        }
    }
}

impl StateFromDigitalInputs<1> for ChassisIntrusion {
    fn from_inputs(inputs: [Level; 1]) -> Self {
        match inputs[0] {
            Level::Low => Self::Intruded,
            Level::High => Self::Normal,
        }
    }
}

pub(crate) static CHASSIS_INTRUSION_CHANGED: Watch<CriticalSectionRawMutex, ChassisIntrusion, 1> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: ChassisIntrusionDetectResources) {
    let mut input: ChassisIntrusionDetector = r.into();

    let mut ticker = Ticker::every(Duration::from_micros(50));

    let tx = CHASSIS_INTRUSION_CHANGED.sender();

    loop {
        ticker.next().await;

        if let Some(state) = input.update() {
            queue_telemetry_message(Payload::Observation(ObservationPayload::ChassisIntrusion(
                (&state).into(),
            )))
            .await;

            tx.send(state);
        }
    }
}
