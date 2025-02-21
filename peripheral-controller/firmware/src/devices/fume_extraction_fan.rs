#[cfg(feature = "telemetry")]
use crate::telemetry::queue_telemetry_message;
use crate::{
    io_helpers::digital_output::{DigitalOutputController, StateToDigitalOutputs},
    FumeExtractionFanResources,
};
use defmt::Format;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
#[cfg(feature = "telemetry")]
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

pub(crate) type FumeExtractionFan = DigitalOutputController<1, FumeExtractionDemand>;

impl From<FumeExtractionFanResources> for FumeExtractionFan {
    fn from(r: FumeExtractionFanResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self::new([output])
    }
}

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
pub(crate) async fn task(r: FumeExtractionFanResources) {
    let mut fume_extraction_fan: FumeExtractionFan = r.into();

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
