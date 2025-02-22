use crate::{
    io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    FumeExtractionModeSwitchResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::Format;
use embassy_rp::gpio::{Input, Level, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};

pub(crate) static FUME_EXTRACTION_MODE_CHANGED: Watch<
    CriticalSectionRawMutex,
    FumeExtractionMode,
    2,
> = Watch::new();

pub(crate) type FumeExtractionModeSwitch =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 1, FumeExtractionMode>;

impl From<FumeExtractionModeSwitchResources> for FumeExtractionModeSwitch {
    fn from(r: FumeExtractionModeSwitchResources) -> Self {
        let input = Input::new(r.switch, Pull::Down);
        Self::new([input])
    }
}

#[derive(Clone, Format)]
pub(crate) enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

impl From<&FumeExtractionMode>
    for hoshiguma_telemetry_protocol::payload::observation::FumeExtractionMode
{
    fn from(value: &FumeExtractionMode) -> Self {
        match value {
            FumeExtractionMode::Automatic => Self::Automatic,
            FumeExtractionMode::OverrideRun => Self::OverrideRun,
        }
    }
}

impl StateFromDigitalInputs<1> for FumeExtractionMode {
    fn from_inputs(inputs: [Level; 1]) -> Self {
        match inputs[0] {
            Level::Low => Self::Automatic,
            Level::High => Self::OverrideRun,
        }
    }
}
