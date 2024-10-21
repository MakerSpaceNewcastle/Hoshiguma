use crate::io_helpers::digital_output::{DigitalOutputController, StateToDigitalOutputs};
#[cfg(feature = "telemetry")]
use crate::telemetry::queue_telemetry_message;
use defmt::Format;
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
#[cfg(feature = "telemetry")]
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

#[macro_export]
macro_rules! init_fume_extraction_fan {
    ($p:expr) => {{
        // Relay output 7
        let output = embassy_rp::gpio::Output::new($p.PIN_21, embassy_rp::gpio::Level::Low);

        $crate::devices::fume_extraction_fan::FumeExtractionFan::new([output])
    }};
}

pub(crate) type FumeExtractionFan = DigitalOutputController<1, FumeExtractionDemand>;

#[derive(Clone, Format)]
pub(crate) enum FumeExtractionDemand {
    Idle,
    Demand,
}

#[cfg(feature = "telemetry")]
impl From<&FumeExtractionDemand>
    for hoshiguma_telemetry_protocol::payload::control::FumeExtractionFan
{
    fn from(value: &FumeExtractionDemand) -> Self {
        match value {
            FumeExtractionDemand::Idle => Self::Idle,
            FumeExtractionDemand::Demand => Self::Demand,
        }
    }
}

impl StateToDigitalOutputs<1> for FumeExtractionDemand {
    fn to_outputs(self) -> [Level; 1] {
        match self {
            Self::Idle => [Level::Low],
            Self::Demand => [Level::High],
        }
    }
}

pub(crate) static FUME_EXTRACTION_FAN: Watch<CriticalSectionRawMutex, FumeExtractionDemand, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(mut fume_extraction_fan: FumeExtractionFan) {
    let mut rx = FUME_EXTRACTION_FAN.receiver().unwrap();

    loop {
        let setting = rx.changed().await;

        #[cfg(feature = "telemetry")]
        queue_telemetry_message(Payload::Control(ControlPayload::FumeExtractionFan(
            (&setting).into(),
        )))
        .await;

        fume_extraction_fan.set(setting);
    }
}
