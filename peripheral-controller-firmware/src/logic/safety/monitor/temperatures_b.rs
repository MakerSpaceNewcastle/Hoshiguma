use super::{temperature_to_state, ObservedSeverity, NEW_MONITOR_STATUS};
use crate::devices::{cooler::COOLER_TEMPERATURES_READ, TemperaturesExt};
use defmt::{unwrap, warn};
use hoshiguma_protocol::{peripheral_controller::types::MonitorKind, types::Severity};

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("temp b mon").await;

    let mut temperatures_rx = unwrap!(COOLER_TEMPERATURES_READ.receiver());
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut sensor_severity = ObservedSeverity::default();
    let mut electronics_severity = ObservedSeverity::default();
    let mut coolant_flow_severity = ObservedSeverity::default();
    let mut heat_exchanger_severity = ObservedSeverity::default();

    let mut sensor_failure_counter = 0;

    loop {
        let state = temperatures_rx.changed().await;

        // Check for faulty sensors
        let new_sensor_severity = if state.any_failed_sensors() {
            sensor_failure_counter += 1;
            warn!(
                "One or more temperature sensors have failed {} times in a row",
                sensor_failure_counter
            );

            if sensor_failure_counter < 3 {
                Severity::Normal
            } else {
                Severity::Warn
            }
        } else {
            sensor_failure_counter = 0;
            Severity::Normal
        };

        // Check for sensor fault
        sensor_severity
            .update_and_async(new_sensor_severity, |severity| async {
                status_tx
                    .publish((MonitorKind::TemperatureSensorFaultB, severity))
                    .await;
            })
            .await;

        // Check electronics temperature
        if let Ok(new_severity) = temperature_to_state(30.0, 35.0, state.onboard) {
            electronics_severity
                .update_and_async(new_severity, |severity| async {
                    status_tx
                        .publish((MonitorKind::CoolerElectronicsOvertemperature, severity))
                        .await;
                })
                .await;
        }

        // Check coolant flow temperature
        if let Ok(new_severity) = temperature_to_state(25.0, 40.0, state.coolant_flow) {
            coolant_flow_severity
                .update_and_async(new_severity, |severity| async {
                    status_tx
                        .publish((MonitorKind::CoolantFlowOvertemperatureB, severity))
                        .await;
                })
                .await;
        }

        // Check heat exchanger temperature
        if let Ok(new_severity) = temperature_to_state(20.0, 25.0, state.heat_exchange_fluid) {
            heat_exchanger_severity
                .update_and_async(new_severity, |severity| async {
                    status_tx
                        .publish((MonitorKind::HeatExchangerOvertemperature, severity))
                        .await;
                })
                .await;
        }
    }
}
