use super::{ObservedSeverity, NEW_MONITOR_STATUS};
use crate::devices::{
    accessories::extraction_airflow_sensor::EXTRACTION_AIRFLOW_SENSOR_READING,
    fume_extraction_fan::FUME_EXTRACTION_FAN,
};
use defmt::{debug, unwrap};
use embassy_futures::select::{select, Either};
use embassy_time::{Duration, Instant};
use hoshiguma_protocol::{
    peripheral_controller::types::{FumeExtractionFan, MonitorKind},
    types::Severity,
};

#[embassy_executor::task]
pub(crate) async fn task() {
    #[cfg(feature = "trace")]
    crate::trace::name_task("extr airflow mon").await;

    let status_tx = unwrap!(NEW_MONITOR_STATUS.publisher());

    let mut extractor_fan_rx = FUME_EXTRACTION_FAN.receiver().unwrap();
    let mut reading_rx = EXTRACTION_AIRFLOW_SENSOR_READING.receiver().unwrap();

    let mut severity = ObservedSeverity::default();

    let mut state = None;
    let mut state_change_time = Instant::now();
    let mut reading = None;

    loop {
        match select(extractor_fan_rx.changed(), reading_rx.changed()).await {
            Either::First(v) => {
                state = Some(v);
                state_change_time = Instant::now();
            }
            Either::Second(v) => reading = Some(v),
        };

        let new_severity = {
            match state {
                Some(FumeExtractionFan::Run) => {
                    let time_fan_running = Instant::now() - state_change_time;
                    if time_fan_running >= FAN_RUNUP_TIME {
                        match reading {
                            Some(Ok(ref reading)) => {
                                if reading.differential_pressure > WARN {
                                    Severity::Normal
                                } else if reading.differential_pressure > CRITICAL {
                                    Severity::Warning
                                } else {
                                    Severity::Critical
                                }
                            }
                            Some(Err(_)) => Severity::Warning,
                            None => Severity::Critical,
                        }
                    } else {
                        debug!("Fan still in runup phase, assume airflow will be OK until it is obviously not");
                        Severity::Normal
                    }
                }
                Some(FumeExtractionFan::Idle) => Severity::Normal,
                None => Severity::Critical,
            }
        };

        severity
            .update_and_async(new_severity, |severity| async {
                status_tx
                    .publish((MonitorKind::ExtractionAirflowInsufficient, severity))
                    .await;
            })
            .await;
    }
}

/// Warning differential pressure in Pa.
const WARN: f32 = 85.0;

/// Critical differential pressure in Pa.
const CRITICAL: f32 = 70.0;

/// Amount of time it typically takes the fan to reach normal operating airflow after it is powered
/// on from stationary.
/// This can be quite conservative as very little fumes will be produced in the first few seconds
/// of a job.
const FAN_RUNUP_TIME: Duration = Duration::from_secs(4);
