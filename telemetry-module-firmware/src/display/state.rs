use crate::{
    network::{NetworkEvent, NETWORK_EVENTS},
    telemetry::TELEMETRY_MESSAGES,
};
use defmt::warn;
use embassy_futures::select::{select, Either};
use embassy_net::StaticConfigV4;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, pubsub::WaitResult, signal::Signal,
};
use hoshiguma_protocol::payload::{
    control::{
        AirAssistPump, ControlPayload, FumeExtractionFan, LaserEnable, MachineEnable, StatusLamp,
    },
    observation::{
        AirAssistDemand, ChassisIntrusion, CoolantResevoirLevelReading, FumeExtractionMode,
        MachinePower, MachineRun, ObservationPayload, Temperatures,
    },
    process::{ActiveAlarms, MachineOperationLockout, ProcessPayload},
    system::{GitRevisionString, SystemMessagePayload},
    Payload,
};

#[derive(Default, Clone)]
pub(crate) struct DisplayDataState {
    // Networking
    pub(crate) ipv4_config: Option<StaticConfigV4>,
    pub(crate) mqtt_broker_connected: bool,

    // Controller system
    pub(crate) controller_git_rev: Option<GitRevisionString>,
    pub(crate) controller_uptime: Option<u64>,

    // Controller observation
    pub(crate) air_assist_demand: Option<AirAssistDemand>,
    pub(crate) chassis_intrusion: Option<ChassisIntrusion>,
    pub(crate) coolant_resevoir_level: Option<CoolantResevoirLevelReading>,
    pub(crate) fume_extraction_mode: Option<FumeExtractionMode>,
    pub(crate) machine_power: Option<MachinePower>,
    pub(crate) machine_run_status: Option<MachineRun>,
    pub(crate) temperatures: Option<Temperatures>,

    // Controller processes
    // TODO: monitors
    pub(crate) alarms: Option<ActiveAlarms>,
    pub(crate) lockout: Option<MachineOperationLockout>,

    // Controller outputs
    pub(crate) air_assist_pump: Option<AirAssistPump>,
    pub(crate) fume_extraction_fan: Option<FumeExtractionFan>,
    pub(crate) laser_enable: Option<LaserEnable>,
    pub(crate) machine_enable: Option<MachineEnable>,
    pub(crate) status_lamp: Option<StatusLamp>,
}

pub(crate) static STATE_CHANGED: Signal<CriticalSectionRawMutex, DisplayDataState> = Signal::new();

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut state = DisplayDataState::default();

    let net_event_rx = NETWORK_EVENTS.receiver();
    let mut telem_rx = TELEMETRY_MESSAGES.subscriber().unwrap();

    loop {
        match select(net_event_rx.receive(), telem_rx.next_message()).await {
            Either::First(event) => match event {
                NetworkEvent::NetworkConnected(ip_config) => {
                    state.ipv4_config = Some(ip_config);
                }
                NetworkEvent::MqttBrokerConnected => {
                    state.mqtt_broker_connected = true;
                }
                NetworkEvent::MqttBrokerDisconnected => {
                    state.mqtt_broker_connected = false;
                }
            },
            Either::Second(msg) => {
                match msg {
                    WaitResult::Lagged(msg_count) => {
                        warn!(
                            "Telemetry message receiver lagged, missed {} messages",
                            msg_count
                        );
                    }
                    WaitResult::Message(msg) => {
                        state.controller_uptime = Some(msg.millis_since_boot);

                        match msg.payload {
                            Payload::System(SystemMessagePayload::Boot(info)) => {
                                state.controller_git_rev = Some(info.git_revision);
                            }
                            Payload::System(SystemMessagePayload::Heartbeat(info)) => {
                                state.controller_git_rev = Some(info.git_revision);
                            }
                            Payload::System(SystemMessagePayload::Panic(_)) => {
                                // Do nothing
                            }
                            Payload::Observation(ObservationPayload::AirAssistDemand(v)) => {
                                state.air_assist_demand = Some(v);
                            }
                            Payload::Observation(ObservationPayload::ChassisIntrusion(v)) => {
                                state.chassis_intrusion = Some(v);
                            }
                            Payload::Observation(ObservationPayload::CoolantResevoirLevel(v)) => {
                                state.coolant_resevoir_level = Some(v);
                            }
                            Payload::Observation(ObservationPayload::FumeExtractionMode(v)) => {
                                state.fume_extraction_mode = Some(v);
                            }
                            Payload::Observation(ObservationPayload::MachinePower(v)) => {
                                state.machine_power = Some(v);
                            }
                            Payload::Observation(ObservationPayload::MachineRun(v)) => {
                                state.machine_run_status = Some(v);
                            }
                            Payload::Observation(ObservationPayload::Temperatures(v)) => {
                                state.temperatures = Some(v);
                            }
                            Payload::Process(ProcessPayload::Monitor(_)) => {
                                // TODO: monitors
                            }
                            Payload::Process(ProcessPayload::Alarms(v)) => {
                                state.alarms = Some(v);
                            }
                            Payload::Process(ProcessPayload::Lockout(v)) => {
                                state.lockout = Some(v);
                            }
                            Payload::Control(ControlPayload::AirAssistPump(v)) => {
                                state.air_assist_pump = Some(v);
                            }
                            Payload::Control(ControlPayload::FumeExtractionFan(v)) => {
                                state.fume_extraction_fan = Some(v);
                            }
                            Payload::Control(ControlPayload::LaserEnable(v)) => {
                                state.laser_enable = Some(v);
                            }
                            Payload::Control(ControlPayload::MachineEnable(v)) => {
                                state.machine_enable = Some(v);
                            }
                            Payload::Control(ControlPayload::StatusLamp(v)) => {
                                state.status_lamp = Some(v);
                            }
                        }
                    }
                }
            }
        };

        STATE_CHANGED.signal(state.clone());
    }
}
