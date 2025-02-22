use crate::{
    io_helpers::{
        digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
        digital_output::{DigitalOutputController, StateToDigitalOutputs},
    },
    telemetry::queue_telemetry_message,
    AirAssistDemandDetectResources, AirAssistPumpResources,
};
use debouncr::{DebouncerStateful, Repeat2};
use defmt::{unwrap, Format};
use embassy_rp::gpio::{Input, Level, Output, Pull};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

pub(crate) type AirAssistDemandDetector =
    DigitalInputStateChangeDetector<DebouncerStateful<u8, Repeat2>, 1, AirAssistDemand>;

impl From<AirAssistDemandDetectResources> for AirAssistDemandDetector {
    fn from(r: AirAssistDemandDetectResources) -> Self {
        let input = Input::new(r.detect, Pull::Down);
        Self::new([input])
    }
}

pub(crate) type AirAssistPump = DigitalOutputController<1, AirAssistDemand>;

impl From<AirAssistPumpResources> for AirAssistPump {
    fn from(r: AirAssistPumpResources) -> Self {
        let output = Output::new(r.relay, Level::Low);
        Self::new([output])
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

impl StateToDigitalOutputs<1> for AirAssistDemand {
    fn to_outputs(self) -> [Level; 1] {
        match self {
            Self::Idle => [Level::Low],
            Self::Demand => [Level::High],
        }
    }
}

pub(crate) static AIR_ASSIST_DEMAND_CHANGED: Watch<CriticalSectionRawMutex, AirAssistDemand, 2> =
    Watch::new();
pub(crate) static AIR_ASSIST_PUMP: Watch<CriticalSectionRawMutex, AirAssistDemand, 2> =
    Watch::new();

#[embassy_executor::task]
pub(crate) async fn pump_task(r: AirAssistPumpResources) {
    let mut air_assist_pump: AirAssistPump = r.into();

    let mut rx = unwrap!(AIR_ASSIST_PUMP.receiver());

    loop {
        let setting = rx.changed().await;

        queue_telemetry_message(Payload::Control(ControlPayload::AirAssistPump(
            (&setting).into(),
        )))
        .await;

        air_assist_pump.set(setting);
    }
}
