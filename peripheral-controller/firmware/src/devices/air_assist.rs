use crate::io_helpers::{
    digital_input::{DigitalInputStateChangeDetector, StateFromDigitalInputs},
    digital_output::{DigitalOutputController, StateToDigitalOutputs},
};
#[cfg(feature = "telemetry")]
use crate::telemetry::queue_telemetry_message;
use defmt::{unwrap, Format};
use embassy_rp::gpio::Level;
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
#[cfg(feature = "telemetry")]
use hoshiguma_telemetry_protocol::payload::{control::ControlPayload, Payload};

#[macro_export]
macro_rules! init_air_assist_demand_detector {
    ($p:expr) => {{
        // Isolated input 4
        let input = embassy_rp::gpio::Input::new($p.PIN_11, embassy_rp::gpio::Pull::Down);

        $crate::devices::air_assist::AirAssistDemandDetector::new([input])
    }};
}

#[macro_export]
macro_rules! init_air_assist_pump {
    ($p:expr) => {{
        // Relay output 6
        let output = embassy_rp::gpio::Output::new($p.PIN_20, embassy_rp::gpio::Level::Low);

        $crate::devices::air_assist::AirAssistPump::new([output])
    }};
}

pub(crate) type AirAssistDemandDetector = DigitalInputStateChangeDetector<1, AirAssistDemand>;
pub(crate) type AirAssistPump = DigitalOutputController<1, AirAssistDemand>;

#[derive(Clone, Format)]
pub(crate) enum AirAssistDemand {
    Idle,
    Demand,
}

#[cfg(feature = "telemetry")]
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

#[cfg(feature = "telemetry")]
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
pub(crate) async fn pump_task(mut air_assist_pump: AirAssistPump) {
    let mut rx = unwrap!(AIR_ASSIST_PUMP.receiver());

    loop {
        let setting = rx.changed().await;

        #[cfg(feature = "telemetry")]
        queue_telemetry_message(Payload::Control(ControlPayload::AirAssistPump(
            (&setting).into(),
        )))
        .await;

        air_assist_pump.set(setting);
    }
}
