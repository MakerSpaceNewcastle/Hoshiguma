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
    let mut reservoir_severity = ObservedSeverity::default();

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

        // Check electronics temperatures
        if let Some(new_severity) = [
            temperature_to_state(35.0, 40.0, state.onboard),
            temperature_to_state(35.0, 40.0, state.internal_ambient),
            temperature_to_state(70.0, 80.0, state.coolant_pump_motor),
        ]
        .iter()
        .flatten()
        .max()
        {
            electronics_severity
                .update_and_async(new_severity.clone(), |severity| async {
                    status_tx
                        .publish((MonitorKind::CoolerElectronicsOvertemperature, severity))
                        .await;
                })
                .await;
        }

        // Check coolant reservoir temperatures
        if let Some(new_severity) = [temperature_to_state(20.0, 25.0, state.reservoir)]
            .iter()
            .flatten()
            .max()
        {
            reservoir_severity
                .update_and_async(new_severity.clone(), |severity| async {
                    status_tx
                        .publish((MonitorKind::CoolantReservoirOvertemperature, severity))
                        .await;
                })
                .await;
        }
    }
}
