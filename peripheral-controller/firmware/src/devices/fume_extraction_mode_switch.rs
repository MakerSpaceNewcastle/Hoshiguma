use crate::io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs};
use defmt::Format;
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};

pub(crate) static FUME_EXTRACTION_MODE_CHANGED: Watch<
    CriticalSectionRawMutex,
    FumeExtractionMode,
    2,
> = Watch::new();

#[macro_export]
macro_rules! init_fume_extraction_mode_switch {
    ($p:expr) => {{
        // Isolated input 5
        let input = embassy_rp::gpio::Input::new($p.PIN_10, embassy_rp::gpio::Pull::Down);

        $crate::devices::fume_extraction_mode_switch::FumeExtractionModeSwitch::new([input])
    }};
}

pub(crate) type FumeExtractionModeSwitch = DigitalInputStateChangeDetector<1, FumeExtractionMode>;

#[derive(Clone, Format)]
pub(crate) enum FumeExtractionMode {
    Automatic,
    OverrideRun,
}

#[cfg(feature = "telemetry")]
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
