mod communication_status;
pub(crate) mod cooler;
pub(crate) mod extraction_airflow_sensor;

use self::cooler::{
    COOLANT_FLOW_READ, COOLANT_RESERVOIR_LEVEL_CHANGED, COOLER_CONTROL_COMMAND,
    COOLER_TEMPERATURES_READ,
};
use super::TemperaturesExt;
use crate::{
    changed::ObservedValue,
    devices::accessories::{
        communication_status::{CommunicationFailureAction, CommunicationStatusReporter},
        extraction_airflow_sensor::EXTRACTION_AIRFLOW_SENSOR_READING,
    },
    telemetry::queue_telemetry_event,
    AccessoriesCommunicationResources,
};
use core::time::Duration as CoreDuration;
use defmt::{debug, unwrap, warn};
use embassy_futures::select::{select3, Either3};
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Ticker, Timer};
use hoshiguma_protocol::{
    accessories::{
        cooler::rpc::{Request as CoolerRequest, Response as CoolerResponse},
        extraction_airflow_sensor::rpc::{
            Request as ExtractionAirflowSensorRequest, Response as ExtractionAirflowSensorResponse,
        },
        rpc::{Request, Response},
        SERIAL_BAUD,
    },
    peripheral_controller::{
        event::{
            ControlEvent as SuperControlEvent, EventKind as SuperEventKind,
            ObservationEvent as SuperObservationEvent,
        },
        types::MonitorKind,
    },
};
use pico_plc_bsp::embassy_rp::{
    bind_interrupts,
    peripherals::UART1,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use static_cell::StaticCell;
use teeny_rpc::{client::Client, transport::embedded::EioTransport};

bind_interrupts!(struct Irqs {
    UART1_IRQ  => BufferedInterruptHandler<UART1>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: AccessoriesCommunicationResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("accessories comm").await;

    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 256;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = SERIAL_BAUD;

    let uart = BufferedUart::new(r.uart, r.tx_pin, r.rx_pin, Irqs, tx_buf, rx_buf, config);

    // Setup RPC client
    let transport = EioTransport::<_, 512>::new(uart);
    let mut client = Client::<_, Request, Response>::new(transport, CoreDuration::from_millis(100));

    let mut update_tick = Ticker::every(Duration::from_secs(1));

    let mut cooler_comm_status = CommunicationStatusReporter::new(
        MonitorKind::CoolerCommunicationFault,
        Duration::from_secs(3),
        Duration::from_secs(10),
    );
    let mut ext_airflow_comm_status = CommunicationStatusReporter::new(
        MonitorKind::ExtractionAirflowSensorFault,
        Duration::from_secs(10),
        Duration::from_secs(30),
    );
    let mut comm_status_check_tick = Ticker::every(Duration::from_secs(1));

    let mut cooler_control_command_rx = unwrap!(COOLER_CONTROL_COMMAND.subscriber());

    let mut coolant_flow = ObservedValue::default();
    let mut temperatures = ObservedValue::default();
    let mut coolant_reservoir_level = ObservedValue::default();
    let coolant_flow_tx = COOLANT_FLOW_READ.sender();
    let temperatures_tx = COOLER_TEMPERATURES_READ.sender();
    let coolant_reservoir_level_tx = COOLANT_RESERVOIR_LEVEL_CHANGED.sender();

    let mut coolant_pump = ObservedValue::default();
    let mut compressor = ObservedValue::default();
    let mut radiator_fan = ObservedValue::default();

    let mut ext_airflow_reading = ObservedValue::default();
    let ext_airflow_reading_tx = EXTRACTION_AIRFLOW_SENSOR_READING.sender();

    loop {
        match select3(
            update_tick.next(),
            cooler_control_command_rx.next_message(),
            comm_status_check_tick.next(),
        )
        .await
        {
            Either3::First(_) => {
                // Cooler
                'cooler_comm_retry: loop {
                    debug!("Cooler update attempt");
                    match client
                        .call(
                            Request::Cooler(CoolerRequest::GetState),
                            core::time::Duration::from_millis(50),
                        )
                        .await
                    {
                        Ok(Response::Cooler(CoolerResponse::GetState(state))) => {
                            debug!("Got state from cooler: {:?}", state);
                            cooler_comm_status.comm_good().await;

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

                            break 'cooler_comm_retry;
                        }
                        Ok(_) => {
                            warn!("Unexpected RPC response");
                            if cooler_comm_status.comm_fail().await
                                == CommunicationFailureAction::GiveUp
                            {
                                break 'cooler_comm_retry;
                            }
                        }
                        Err(e) => {
                            warn!("RPC error: {}", e);
                            if cooler_comm_status.comm_fail().await
                                == CommunicationFailureAction::GiveUp
                            {
                                break 'cooler_comm_retry;
                            }
                        }
                    }
                }

                // Extraction airflow sensor
                'airflow_sensor_comm_retry: loop {
                    debug!("Extraction airflow sensor update attempt");
                    match client
                        .call(
                            Request::ExtractionAirflowSensor(
                                ExtractionAirflowSensorRequest::GetMeasurement,
                            ),
                            core::time::Duration::from_millis(50),
                        )
                        .await
                    {
                        Ok(Response::ExtractionAirflowSensor(
                            ExtractionAirflowSensorResponse::GetMeasurement(measurement),
                        )) => {
                            debug!(
                                "Got measurement from extraction airflow sensor: {:?}",
                                measurement
                            );
                            ext_airflow_comm_status.comm_good().await;

                            ext_airflow_reading
                                .update_and_async(measurement, |value| async {
                                    ext_airflow_reading_tx.send(value.clone());
                                    queue_telemetry_event(SuperEventKind::Observation(
                                        SuperObservationEvent::ExtractionAirflowMeasurement(value),
                                    ))
                                    .await;
                                })
                                .await;

                            break 'airflow_sensor_comm_retry;
                        }
                        Ok(_) => {
                            warn!("Unexpected RPC response");
                            if ext_airflow_comm_status.comm_fail().await
                                == CommunicationFailureAction::GiveUp
                            {
                                break 'airflow_sensor_comm_retry;
                            }
                        }
                        Err(e) => {
                            warn!("RPC error: {}", e);
                            if ext_airflow_comm_status.comm_fail().await
                                == CommunicationFailureAction::GiveUp
                            {
                                break 'airflow_sensor_comm_retry;
                            }
                        }
                    }
                }
            }
            Either3::Second(WaitResult::Message(cmd)) => {
                let request: CoolerRequest = cmd.into();

                'cmd_send: for attempt in 0..5 {
                    match client
                        .call(
                            Request::Cooler(request.clone()),
                            core::time::Duration::from_millis(50),
                        )
                        .await
                    {
                        Ok(_) => {
                            cooler_comm_status.comm_good().await;
                            break 'cmd_send;
                        }
                        Err(e) => {
                            warn!("RPC error: {} (attempt {})", e, attempt + 1);
                            cooler_comm_status.comm_fail().await;
                            Timer::after_millis(50).await;
                        }
                    }
                }
            }
            Either3::Second(WaitResult::Lagged(msg_count)) => {
                panic!("Subscriber lagged, losing {} messages", msg_count);
            }
            Either3::Third(_) => {
                cooler_comm_status.evaluate().await;
                ext_airflow_comm_status.evaluate().await;
            }
        }
    }
}
