use super::TemperaturesExt;
use crate::{
    logic::safety::monitor::{ObservedSeverity, NEW_MONITOR_STATUS},
    telemetry::queue_telemetry_event,
    CoolerCommunicationResources,
};
use defmt::{info, unwrap, warn, Format};
use embassy_futures::select::{select3, Either3};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART1,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, WaitResult},
    watch::Watch,
};
use embassy_time::{Duration, Instant, Ticker, Timer};
use hoshiguma_protocol::{
    cooler::{
        event::{ControlEvent, EventKind, ObservationEvent},
        rpc::{Request, Response},
        types::{
            Compressor, CoolantFlow, CoolantPump, HeaderTankCoolantLevelReading,
            HeatExchangeFluidLevel, RadiatorFan, Stirrer, Temperatures,
        },
    },
    peripheral_controller::{
        event::{
            ControlEvent as SuperControlEvent, EventKind as SuperEventKind,
            ObservationEvent as SuperObservationEvent,
        },
        types::MonitorKind,
    },
    types::Severity,
};
use static_cell::StaticCell;
use teeny_rpc::{client::Client, transport::embedded::EioTransport};

#[derive(Debug, Clone, Format)]
pub(crate) enum CoolerControlCommand {
    SetRadiatorFan(RadiatorFan),
    SetCompressor(Compressor),
    SetStirrer(Stirrer),
    SetCoolantPump(CoolantPump),
}

impl From<CoolerControlCommand> for Request {
    fn from(cmd: CoolerControlCommand) -> Self {
        match cmd {
            CoolerControlCommand::SetRadiatorFan(radiator_fan) => {
                Self::SetRadiatorFan(radiator_fan)
            }
            CoolerControlCommand::SetCompressor(compressor) => Self::SetCompressor(compressor),
            CoolerControlCommand::SetStirrer(stirrer) => Self::SetStirrer(stirrer),
            CoolerControlCommand::SetCoolantPump(coolant_pump) => {
                Self::SetCoolantPump(coolant_pump)
            }
        }
    }
}

pub(crate) static COOLER_CONTROL_COMMAND: PubSubChannel<
    CriticalSectionRawMutex,
    CoolerControlCommand,
    8,
    1,
    2,
> = PubSubChannel::new();

pub(crate) static COOLANT_FLOW_READ: Watch<CriticalSectionRawMutex, CoolantFlow, 1> = Watch::new();

pub(crate) static COOLER_TEMPERATURES_READ: Watch<CriticalSectionRawMutex, Temperatures, 1> =
    Watch::new();

pub(crate) static HEADER_TANK_COOLANT_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    HeaderTankCoolantLevelReading,
    1,
> = Watch::new();

pub(crate) static HEAT_EXCHANGER_FLUID_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    HeatExchangeFluidLevel,
    1,
> = Watch::new();

bind_interrupts!(struct Irqs {
    UART1_IRQ  => BufferedInterruptHandler<UART1>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: CoolerCommunicationResources) {
    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 256;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = hoshiguma_protocol::peripheral_controller::SERIAL_BAUD;

    let uart = BufferedUart::new(r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buf, rx_buf, config);

    // Setup RPC client
    let transport = EioTransport::new(uart);
    let mut client = Client::<_, Request, Response>::new(transport);

    let mut comm_status = CommunicationStatusReporter::default();
    let mut comm_status_check_tick = Ticker::every(Duration::from_secs(1));

    let mut control_command_rx = unwrap!(COOLER_CONTROL_COMMAND.subscriber());

    const SHORT_EVENT_POLL: Duration = Duration::from_millis(50);
    const LONG_EVENT_POLL: Duration = Duration::from_millis(500);

    let mut event_poll_interval = LONG_EVENT_POLL;

    let coolant_flow_tx = COOLANT_FLOW_READ.sender();
    let temperatures_tx = COOLER_TEMPERATURES_READ.sender();
    let heat_exchanger_fluid_level_tx = HEAT_EXCHANGER_FLUID_LEVEL_CHANGED.sender();
    let header_tank_level_tx = HEADER_TANK_COOLANT_LEVEL_CHANGED.sender();

    loop {
        match select3(
            Timer::after(event_poll_interval),
            control_command_rx.next_message(),
            comm_status_check_tick.next(),
        )
        .await
        {
            Either3::First(_) => {
                match client
                    .call(
                        Request::GetOldestEvent,
                        core::time::Duration::from_millis(50),
                    )
                    .await
                {
                    Ok(Response::GetOldestEvent(Some(event))) => {
                        info!("Got event from cooler: {:?}", event);
                        comm_status.comm_good().await;

                        match event.kind {
                            EventKind::Boot(info) => {
                                queue_telemetry_event(SuperEventKind::CoolerBoot(info)).await;
                            }
                            EventKind::Observation(ObservationEvent::CoolantFlow(v)) => {
                                coolant_flow_tx.send(v.clone());
                                queue_telemetry_event(SuperEventKind::Observation(
                                    SuperObservationEvent::CoolantFlow(v),
                                ))
                                .await;
                            }
                            EventKind::Observation(ObservationEvent::Temperatures(v)) => {
                                temperatures_tx.send(v.clone());
                                queue_telemetry_event(SuperEventKind::Observation(
                                    SuperObservationEvent::TemperaturesB(v),
                                ))
                                .await;
                            }
                            EventKind::Observation(ObservationEvent::HeatExchangeFluidLevel(v)) => {
                                heat_exchanger_fluid_level_tx.send(v.clone());
                                queue_telemetry_event(SuperEventKind::Observation(
                                    SuperObservationEvent::HeatExchangerFluidLevel(v),
                                ))
                                .await;
                            }
                            EventKind::Observation(ObservationEvent::HeaderTankCoolantLevel(v)) => {
                                header_tank_level_tx.send(v.clone());
                                queue_telemetry_event(SuperEventKind::Observation(
                                    SuperObservationEvent::CoolantHeaderTankLevel(v),
                                ))
                                .await;
                            }
                            EventKind::Control(ControlEvent::Stirrer(v)) => {
                                queue_telemetry_event(SuperEventKind::Control(
                                    SuperControlEvent::CoolerStirrer(v),
                                ))
                                .await;
                            }
                            EventKind::Control(ControlEvent::Compressor(v)) => {
                                queue_telemetry_event(SuperEventKind::Control(
                                    SuperControlEvent::CoolerCompressor(v),
                                ))
                                .await;
                            }
                            EventKind::Control(ControlEvent::RadiatorFan(v)) => {
                                queue_telemetry_event(SuperEventKind::Control(
                                    SuperControlEvent::CoolerRadiatorFan(v),
                                ))
                                .await;
                            }
                            EventKind::Control(ControlEvent::CoolantPump(v)) => {
                                queue_telemetry_event(SuperEventKind::Control(
                                    SuperControlEvent::CoolantPump(v),
                                ))
                                .await;
                            }
                        }

                        event_poll_interval = SHORT_EVENT_POLL;
                    }
                    Ok(Response::GetOldestEvent(None)) => {
                        comm_status.comm_good().await;
                        event_poll_interval = LONG_EVENT_POLL;
                    }
                    Ok(_) => {
                        warn!("Unexpected RPC response");
                        comm_status.comm_fail().await;
                    }
                    Err(e) => {
                        warn!("RPC error: {}", e);
                        comm_status.comm_fail().await;
                    }
                }
            }
            Either3::Second(WaitResult::Message(cmd)) => {
                let request: Request = cmd.into();

                'cmd_send: for attempt in 0..5 {
                    match client
                        .call(request.clone(), core::time::Duration::from_millis(50))
                        .await
                    {
                        Ok(_) => {
                            comm_status.comm_good().await;
                            break 'cmd_send;
                        }
                        Err(e) => {
                            warn!("RPC error: {} (attempt {})", e, attempt + 1);
                            comm_status.comm_fail().await;
                            Timer::after_millis(50).await;
                        }
                    }
                }
            }
            Either3::Second(WaitResult::Lagged(msg_count)) => {
                panic!("Subscriber lagged, losing {} messages", msg_count);
            }
            Either3::Third(_) => {
                comm_status.evaluate().await;
            }
        }
    }
}

enum CommunicationStatus {
    Ok { last: Instant },
    Failed { since: Instant },
}

struct CommunicationStatusReporter {
    status: CommunicationStatus,
    severity: ObservedSeverity,
    monitor_tx: Publisher<'static, CriticalSectionRawMutex, (MonitorKind, Severity), 8, 1, 8>,
}

impl Default for CommunicationStatusReporter {
    fn default() -> Self {
        let monitor_tx = unwrap!(NEW_MONITOR_STATUS.publisher());
        Self {
            status: CommunicationStatus::Failed {
                since: Instant::now(),
            },
            severity: ObservedSeverity::default(),
            monitor_tx,
        }
    }
}

impl CommunicationStatusReporter {
    async fn comm_good(&mut self) {
        self.status = CommunicationStatus::Ok {
            last: Instant::now(),
        };

        self.evaluate().await;
    }

    async fn comm_fail(&mut self) {
        self.status = match self.status {
            CommunicationStatus::Ok { last: _ } => CommunicationStatus::Failed {
                since: Instant::now(),
            },
            CommunicationStatus::Failed { since } => CommunicationStatus::Failed { since },
        };

        self.evaluate().await;
    }

    async fn evaluate(&mut self) {
        const WARN_TIMEOUT: Duration = Duration::from_secs(3);
        const CRITICAL_TIMEOUT: Duration = Duration::from_secs(10);

        let severity = match self.status {
            CommunicationStatus::Ok { last } => {
                if Instant::now().saturating_duration_since(last) > WARN_TIMEOUT {
                    self.status = CommunicationStatus::Failed {
                        since: Instant::now(),
                    };
                    Severity::Warn
                } else {
                    Severity::Normal
                }
            }
            CommunicationStatus::Failed { since } => {
                if Instant::now().saturating_duration_since(since) > CRITICAL_TIMEOUT {
                    Severity::Critical
                } else {
                    Severity::Warn
                }
            }
        };

        self.severity
            .update_and_async(severity, |severity| async {
                self.monitor_tx
                    .publish((MonitorKind::CoolerCommunicationFault, severity))
                    .await;
            })
            .await;
    }
}

impl TemperaturesExt for Temperatures {
    fn any_failed_sensors(&self) -> bool {
        let sensors = [
            &self.onboard,
            &self.coolant_flow,
            &self.coolant_mid,
            &self.coolant_return,
            &self.heat_exchange_fluid,
            &self.heat_exchanger_loop,
        ];

        sensors.iter().any(|i| i.is_err())
    }
}
