use crate::{telemetry::queue_telemetry_message, FumeExtractionFanResources};
use defmt::Format;
use embassy_rp::gpio::{Level, Output};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::payload::{control::ControlPayload, Payload};

#[derive(Clone, Format)]
pub(crate) enum FumeExtractionDemand {
    Idle,
    Demand,
}

impl From<&FumeExtractionDemand> for hoshiguma_protocol::payload::control::FumeExtractionFan {
    fn from(value: &FumeExtractionDemand) -> Self {
        match value {
            FumeExtractionDemand::Idle => Self::Idle,
            FumeExtractionDemand::Demand => Self::Demand,
        }
    }
}

pub(crate) static FUME_EXTRACTION_FAN: Watch<CriticalSectionRawMutex, FumeExtractionDemand, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn task(r: FumeExtractionFanResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = FUME_EXTRACTION_FAN.receiver().unwrap();

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_message(Payload::Control(ControlPayload::FumeExtractionFan(
            (&setting).into(),
        )))
        .await;

        // Set relay output
        let level = match setting {
            FumeExtractionDemand::Idle => Level::Low,
            FumeExtractionDemand::Demand => Level::High,
        };
        output.set_level(level);
    }
}
