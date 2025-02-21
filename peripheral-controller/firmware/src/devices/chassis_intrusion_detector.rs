use crate::{
    io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    ChassisIntrusionDetectResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};

pub(crate) static CHASSIS_INTRUSION_CHANGED: Watch<CriticalSectionRawMutex, ChassisIntrusion, 1> =
    Watch::new();

pub(crate) type ChassisIntrusionDetector =
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

#[cfg(feature = "telemetry")]
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
