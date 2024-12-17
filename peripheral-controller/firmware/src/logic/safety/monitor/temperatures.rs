use super::{Monitor, MonitorState, MonitorStatus, NEW_MONITOR_STATUS};
use crate::{
    changed::Changed,
    devices::temperature_sensors::{TemperatureReading, TEMPERATURES_READ},
};
use defmt::{debug, unwrap, warn};

fn temperature_to_state(
    warn: f32,
    critical: f32,
    temperature: TemperatureReading,
) -> Result<MonitorState, ()> {
    if let Ok(temperature) = temperature {
        Ok(if temperature >= critical {
            warn!(
                "Temperature {} is above critical threshold of {}",
                temperature, critical
            );
            MonitorState::Critical
        } else if temperature >= warn {
            warn!(
                "Temperature {} is above warning threshold of {}",
                temperature, warn
            );
            MonitorState::Warn
        } else {
            debug!("Temperature {} is normal", temperature);
            MonitorState::Normal
        })
    } else {
        warn!("Asked to check temperature of a sensor that failed to be read");
        Err(())
    }
}

#[embassy_executor::task]
pub(crate) async fn task() {
    let mut rx = unwrap!(TEMPERATURES_READ.receiver());

    let mut sensor_status = MonitorStatus::new(Monitor::TemperatureSensorFault);
    let mut coolant_flow_status = MonitorStatus::new(Monitor::CoolantFlowTemperature);
    let mut coolant_resevoir_status = MonitorStatus::new(Monitor::CoolantResevoirTemperature);

    let mut sensor_failure_counter = 0;

    loop {
        let state = rx.changed().await;

        // Check for faulty sensors
        let sensor_state = match state.overall_result() {
            Ok(_) => {
                sensor_failure_counter = 0;
                MonitorState::Normal
            }
            Err(_) => {
                sensor_failure_counter += 1;
                warn!(
                    "One or more temperature sensors have failed {} times in a row",
                    sensor_failure_counter
                );

                if sensor_failure_counter < 3 {
                    MonitorState::Normal
                } else {
                    MonitorState::Warn
                }
            }
        };

        if sensor_status.refresh(sensor_state) == Changed::Yes {
            NEW_MONITOR_STATUS.send(sensor_status.clone()).await;
        }

        // Check coolant flow temperature
        if let Ok(coolant_flow) = temperature_to_state(25.0, 40.0, state.coolant_flow) {
            if coolant_flow_status.refresh(coolant_flow) == Changed::Yes {
                NEW_MONITOR_STATUS.send(coolant_flow_status.clone()).await;
            }
        }

        // Check coolant resevoir temperatures
        {
            let mut resevoir_state = MonitorState::Normal;

            if let Ok(coolant_resevoir_top) =
                temperature_to_state(28.0, 40.0, state.coolant_resevoir_top)
            {
                resevoir_state.upgrade(coolant_resevoir_top);
            }

            if let Ok(coolant_resevoir_bottom) =
                temperature_to_state(28.0, 40.0, state.coolant_resevoir_bottom)
            {
                resevoir_state.upgrade(coolant_resevoir_bottom);
            }

            if coolant_resevoir_status.refresh(resevoir_state) == Changed::Yes {
                NEW_MONITOR_STATUS
                    .send(coolant_resevoir_status.clone())
                    .await;
            }
        }
    }
}
