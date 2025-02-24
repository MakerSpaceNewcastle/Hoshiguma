use crate::{
    io_helpers::digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    telemetry::queue_telemetry_message,
    AirAssistDemandDetectResources, AirAssistPumpResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::{unwrap, Format};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::{Duration, Ticker};
use hoshiguma_telemetry_protocol::payload::{
    control::ControlPayload, observation::ObservationPayload, Payload,
};

type AirAssistDemandDetector =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 1, AirAssistDemand>;

impl From<AirAssistDemandDetectResources> for AirAssistDemandDetector {
    fn from(r: AirAssistDemandDetectResources) -> Self {
        let input = Input::new(r.detect, Pull::Down);
        Self::new([input])
    }
}

#[derive(Clone, Format)]
pub(crate) enum AirAssistDemand {
    Idle,
    Demand,
}

impl From<&AirAssistDemand>
    for hoshiguma_telemetry_protocol::payload::observation::AirAssistDemand
{
    fn from(demand: &AirAssistDemand) -> Self {
        match demand {
            AirAssistDemand::Idle => Self::Idle,
            AirAssistDemand::Demand => Self::Demand,
        }
    }
}

impl From<&AirAssistDemand> for hoshiguma_telemetry_protocol::payload::control::AirAssistPump {
    fn from(value: &AirAssistDemand) -> Self {
        match value {
            AirAssistDemand::Idle => Self::Idle,
            AirAssistDemand::Demand => Self::Demand,
        }
    }
}

impl StateFromDigitalInputs<1> for AirAssistDemand {
    fn from_inputs(inputs: [Level; 1]) -> Self {
        match inputs[0] {
            Level::Low => Self::Idle,
            Level::High => Self::Demand,
        }
    }
}

pub(crate) static AIR_ASSIST_DEMAND_CHANGED: Watch<CriticalSectionRawMutex, AirAssistDemand, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn demand_task(r: AirAssistDemandDetectResources) {
    let mut input: AirAssistDemandDetector = r.into();

    let mut ticker = Ticker::every(Duration::from_millis(100));

    let tx = AIR_ASSIST_DEMAND_CHANGED.sender();

    loop {
        ticker.next().await;

        if let Some(state) = input.update() {
            queue_telemetry_message(Payload::Observation(ObservationPayload::AirAssistDemand(
                (&state).into(),
            )))
            .await;

            tx.send(state);
        }
    }
}

pub(crate) static AIR_ASSIST_PUMP: Watch<CriticalSectionRawMutex, AirAssistDemand, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn pump_task(r: AirAssistPumpResources) {
    let mut output = Output::new(r.relay, Level::Low);
    let mut rx = unwrap!(AIR_ASSIST_PUMP.receiver());

    loop {
        // Wait for a new setting
        let setting = rx.changed().await;

        // Send telemetry update
        queue_telemetry_message(Payload::Control(ControlPayload::AirAssistPump(
            (&setting).into(),
        )))
        .await;

        // Set relay output
        let level = match setting {
            AirAssistDemand::Idle => Level::Low,
            AirAssistDemand::Demand => Level::High,
        };
        output.set_level(level);
    }
}
