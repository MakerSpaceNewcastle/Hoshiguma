use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_protocol::accessories::extraction_airflow_sensor::types::FallibleMeasurement;

pub(crate) static EXTRACTION_AIRFLOW_SENSOR_READING: Watch<
    CriticalSectionRawMutex,
    FallibleMeasurement,
    1,
> = Watch::new();
