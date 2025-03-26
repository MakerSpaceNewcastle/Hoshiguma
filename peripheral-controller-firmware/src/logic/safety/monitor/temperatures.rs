use super::NEW_MONITOR_STATUS;
use crate::{
    changed::{checked_set, Changed},
    devices::temperature_sensors::{TemperaturesExt, TEMPERATURES_READ},
};
use defmt::{debug, unwrap, warn};
use hoshiguma_protocol::{
    peripheral_controller::types::{MonitorKind, TemperatureReading},
    types::Severity,
};

fn temperature_to_state(
    warn: f32,
    critical: f32,
    temperature: TemperatureReading,
) -> Result<Severity, ()> {
    if let Ok(temperature) = temperature {
        Ok(if temperature >= critical {
            warn!(
                "Temperature {} is above critical threshold of {}",
                temperature, critical
            );
            Severity::Critical
        } else if temperature >= warn {
            warn!(
                "Temperature {} is above warning threshold of {}",
                temperature, warn
            );
            Severity::Warn
        } else {
            debug!("Temperature {} is normal", temperature);
            Severity::Normal
        })
    } else {
        warn!("Asked to check temperature of a sensor that failed to be read");
        Err(())
    }
}

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut temperatures_rx = unwrap!(TEMPERATURES_READ.receiver());
    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut sensor_severity = Severity::Critical;
    let mut coolant_flow_severity = Severity::Critical;
    let mut coolant_resevoir_severity = Severity::Critical;

    let mut sensor_failure_counter = 0;

    loop {
        let state = temperatures_rx.changed().await;

        // Check for faulty sensors
        let new_sensor_severity = match state.overall_result() {
            Ok(_) => {
                sensor_failure_counter = 0;
                Severity::Normal
            }
            Err(_) => {
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
            }
        };

        if checked_set(&mut sensor_severity, new_sensor_severity) == Changed::Yes {
            status_tx
                .publish((MonitorKind::TemperatureSensorFault, sensor_severity.clone()))
                .await;
        }

        // Check coolant flow temperature
        if let Ok(new_severity) = temperature_to_state(25.0, 40.0, state.coolant_flow) {
            if checked_set(&mut coolant_flow_severity, new_severity) == Changed::Yes {
                status_tx
                    .publish((
                        MonitorKind::CoolantFlowTemperature,
                        coolant_flow_severity.clone(),
                    ))
                    .await;
            }
        }

        // Check coolant resevoir temperatures
        {
            let mut new_severity = Severity::Normal;

            if let Ok(coolant_resevoir_top) =
                temperature_to_state(28.0, 40.0, state.coolant_resevoir_top)
            {
                new_severity = core::cmp::max(new_severity, coolant_resevoir_top);
            }

            if let Ok(coolant_resevoir_bottom) =
                temperature_to_state(28.0, 40.0, state.coolant_resevoir_bottom)
            {
                new_severity = core::cmp::max(new_severity, coolant_resevoir_bottom);
            }

            if checked_set(&mut coolant_resevoir_severity, new_severity) == Changed::Yes {
                status_tx
                    .publish((
                        MonitorKind::CoolantResevoirTemperature,
                        coolant_resevoir_severity.clone(),
                    ))
                    .await;
            }
        }
    }
}
