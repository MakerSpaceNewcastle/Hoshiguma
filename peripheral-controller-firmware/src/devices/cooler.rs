use super::TemperaturesExt;
use crate::{
    changed::ObservedValue,
    logic::safety::monitor::{ObservedSeverity, NEW_MONITOR_STATUS},
    telemetry::queue_telemetry_event,
    CoolerCommunicationResources,
};
use core::time::Duration as CoreDuration;
use defmt::{debug, unwrap, warn, Format};
use embassy_futures::select::{select3, Either3};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, Publisher, WaitResult},
    watch::Watch,
};
use embassy_time::{Duration, Instant, Ticker, Timer};
use hoshiguma_protocol::{
    cooler::{
        rpc::{Request, Response},
        types::{
            CompressorState, CoolantFlow, CoolantPumpState, CoolantReservoirLevel,
            RadiatorFanState, Temperatures,
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
use pico_plc_bsp::embassy_rp::{
    bind_interrupts,
    peripherals::UART1,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use static_cell::StaticCell;
use teeny_rpc::{client::Client, transport::embedded::EioTransport};

#[derive(Debug, Clone, Format)]
pub(crate) enum CoolerControlCommand {
    RadiatorFan(RadiatorFanState),
    Compressor(CompressorState),
    CoolantPump(CoolantPumpState),
}

impl From<CoolerControlCommand> for Request {
    fn from(cmd: CoolerControlCommand) -> Self {
        match cmd {
            CoolerControlCommand::RadiatorFan(radiator_fan) => Self::SetRadiatorFan(radiator_fan),
            CoolerControlCommand::Compressor(compressor) => Self::SetCompressor(compressor),
            CoolerControlCommand::CoolantPump(coolant_pump) => Self::SetCoolantPump(coolant_pump),
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

pub(crate) static COOLER_TEMPERATURES_READ: Watch<CriticalSectionRawMutex, Temperatures, 2> =
    Watch::new();

pub(crate) static COOLANT_RESEVOIR_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    CoolantReservoirLevel,
    1,
> = Watch::new();

bind_interrupts!(struct Irqs {
    UART1_IRQ  => BufferedInterruptHandler<UART1>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: CoolerCommunicationResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("cooler comm").await;

    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 256;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = hoshiguma_protocol::peripheral_controller::SERIAL_BAUD;

    let uart = BufferedUart::new(r.uart, r.tx_pin, r.rx_pin, Irqs, tx_buf, rx_buf, config);

    // Setup RPC client
    let transport = EioTransport::<_, 512>::new(uart);
    let mut client = Client::<_, Request, Response>::new(transport, CoreDuration::from_millis(100));

    let mut get_state_tick = Ticker::every(Duration::from_secs(1));

    let mut comm_status = CommunicationStatusReporter::default();
    let mut comm_status_check_tick = Ticker::every(Duration::from_secs(1));

    let mut control_command_rx = unwrap!(COOLER_CONTROL_COMMAND.subscriber());

    let mut coolant_flow = ObservedValue::default();
    let mut temperatures = ObservedValue::default();
    let mut coolant_reservoir_level = ObservedValue::default();
    let coolant_flow_tx = COOLANT_FLOW_READ.sender();
    let temperatures_tx = COOLER_TEMPERATURES_READ.sender();
    let coolant_reservoir_level_tx = COOLANT_RESEVOIR_LEVEL_CHANGED.sender();

    let mut coolant_pump = ObservedValue::default();
    let mut compressor = ObservedValue::default();
    let mut radiator_fan = ObservedValue::default();

    loop {
        match select3(
            get_state_tick.next(),
            control_command_rx.next_message(),
            comm_status_check_tick.next(),
        )
        .await
        {
            Either3::First(_) => {
                match client
                    .call(Request::GetState, core::time::Duration::from_millis(50))
                    .await
                {
                    Ok(Response::GetState(state)) => {
                        debug!("Got state from cooler: {:?}", state);
                        comm_status.comm_good().await;

                        coolant_flow
                            .update_and_async(state.coolant_flow_rate, |value| async {
                                coolant_flow_tx.send(value.clone());
                                queue_telemetry_event(SuperEventKind::Observation(
                                    SuperObservationEvent::CoolantFlow(value),
                                ))
                                .await;
                            })
                            .await;

                        temperatures
                            .update_and_async(state.temperatures, |value| async {
                                temperatures_tx.send(value.clone());
                                queue_telemetry_event(SuperEventKind::Observation(
                                    SuperObservationEvent::TemperaturesB(value),
                                ))
                                .await;
                            })
                            .await;

                        coolant_reservoir_level
                            .update_and_async(state.coolant_reservoir_level, |value| async {
                                coolant_reservoir_level_tx.send(value.clone());
                                queue_telemetry_event(SuperEventKind::Observation(
                                    SuperObservationEvent::CoolantReservoirLevel(value),
                                ))
                                .await;
                            })
                            .await;

                        coolant_pump
                            .update_and_async(state.coolant_pump, |value| async {
                                queue_telemetry_event(SuperEventKind::Control(
                                    SuperControlEvent::CoolantPump(value),
                                ))
                                .await
                            })
                            .await;

                        compressor
                            .update_and_async(state.compressor, |value| async {
                                queue_telemetry_event(SuperEventKind::Control(
                                    SuperControlEvent::CoolerCompressor(value),
                                ))
                                .await
                            })
                            .await;

                        radiator_fan
                            .update_and_async(state.radiator_fan, |value| async {
                                queue_telemetry_event(SuperEventKind::Control(
                                    SuperControlEvent::CoolerRadiatorFan(value),
                                ))
                                .await
                            })
                            .await;
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
            &self.internal_ambient,
            &self.reservoir_evaporator_coil,
            &self.reservoir_left_side,
            &self.reservoir_right_side,
            &self.coolant_pump_motor,
        ];

        sensors.iter().any(|i| i.is_err())
    }
}
