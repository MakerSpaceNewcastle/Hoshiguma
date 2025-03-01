use crate::{
    polled_input::PolledInput, telemetry::queue_telemetry_message, AirAssistDemandDetectResources,
    AirAssistPumpResources,
};
use defmt::{unwrap, Format};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use embassy_time::Duration;
use hoshiguma_protocol::payload::{
    control::ControlPayload, observation::ObservationPayload, Payload,
};

#[derive(Clone, Format)]
pub(crate) enum AirAssistDemand {
    Idle,
    Demand,
}

impl From<&AirAssistDemand> for hoshiguma_protocol::payload::observation::AirAssistDemand {
    fn from(demand: &AirAssistDemand) -> Self {
        match demand {
            AirAssistDemand::Idle => Self::Idle,
            AirAssistDemand::Demand => Self::Demand,
        }
    }
}

impl From<&AirAssistDemand> for hoshiguma_protocol::payload::control::AirAssistPump {
    fn from(value: &AirAssistDemand) -> Self {
        match value {
            AirAssistDemand::Idle => Self::Idle,
            AirAssistDemand::Demand => Self::Demand,
        }
    }
}

pub(crate) static AIR_ASSIST_DEMAND_CHANGED: Watch<CriticalSectionRawMutex, AirAssistDemand, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn demand_task(r: AirAssistDemandDetectResources) {
    let pin = Input::new(r.detect, Pull::Down);
    let mut input = PolledInput::new(pin, Duration::from_millis(10));

    let tx = AIR_ASSIST_DEMAND_CHANGED.sender();

    loop {
        let state = input.wait_for_change().await;

        let state = match state {
            Level::Low => AirAssistDemand::Idle,
            Level::High => AirAssistDemand::Demand,
        };

        queue_telemetry_message(Payload::Observation(ObservationPayload::AirAssistDemand(
            (&state).into(),
        )))
        .await;

        tx.send(state);
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
