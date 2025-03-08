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
use heapless::String;
use hoshiguma_protocol::peripheral_controller::{
    event::{ControlEvent, EventKind, ObservationEvent, ProcessEvent},
    types::{
        ActiveAlarms, AirAssistDemand, AirAssistPump, ChassisIntrusion,
        CoolantResevoirLevelReading, FumeExtractionFan, FumeExtractionMode, LaserEnable,
        MachineEnable, MachineOperationLockout, MachinePower, MachineRun, StatusLamp, Temperatures,
    },
};

#[derive(Default, Clone)]
pub(crate) struct DisplayDataState {
    // Networking
    pub(crate) ipv4_config: Option<StaticConfigV4>,
    pub(crate) mqtt_broker_connected: bool,

    // Controller system
    pub(crate) controller_git_rev: Option<String<20>>,
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
            Either::Second(event) => {
                match event {
                    WaitResult::Lagged(msg_count) => {
                        warn!(
                            "Telemetry message receiver lagged, missed {} messages",
                            msg_count
                        );
                    }
                    WaitResult::Message(event) => {
                        state.controller_uptime = Some(event.timestamp_milliseconds);

                        match event.kind {
                            EventKind::Boot(info) => {
                                state.controller_git_rev = Some(info.git_revision);
                            }
                            EventKind::Observation(ObservationEvent::AirAssistDemand(v)) => {
                                state.air_assist_demand = Some(v);
                            }
                            EventKind::Observation(ObservationEvent::ChassisIntrusion(v)) => {
                                state.chassis_intrusion = Some(v);
                            }
                            EventKind::Observation(ObservationEvent::CoolantResevoirLevel(v)) => {
                                state.coolant_resevoir_level = Some(v);
                            }
                            EventKind::Observation(ObservationEvent::FumeExtractionMode(v)) => {
                                state.fume_extraction_mode = Some(v);
                            }
                            EventKind::Observation(ObservationEvent::MachinePower(v)) => {
                                state.machine_power = Some(v);
                            }
                            EventKind::Observation(ObservationEvent::MachineRun(v)) => {
                                state.machine_run_status = Some(v);
                            }
                            EventKind::Observation(ObservationEvent::Temperatures(v)) => {
                                state.temperatures = Some(v);
                            }
                            EventKind::Process(ProcessEvent::Monitor(_)) => {
                                // TODO: monitors
                            }
                            EventKind::Process(ProcessEvent::Alarms(v)) => {
                                state.alarms = Some(v);
                            }
                            EventKind::Process(ProcessEvent::Lockout(v)) => {
                                state.lockout = Some(v);
                            }
                            EventKind::Control(ControlEvent::AirAssistPump(v)) => {
                                state.air_assist_pump = Some(v);
                            }
                            EventKind::Control(ControlEvent::FumeExtractionFan(v)) => {
                                state.fume_extraction_fan = Some(v);
                            }
                            EventKind::Control(ControlEvent::LaserEnable(v)) => {
                                state.laser_enable = Some(v);
                            }
                            EventKind::Control(ControlEvent::MachineEnable(v)) => {
                                state.machine_enable = Some(v);
                            }
                            EventKind::Control(ControlEvent::StatusLamp(v)) => {
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
