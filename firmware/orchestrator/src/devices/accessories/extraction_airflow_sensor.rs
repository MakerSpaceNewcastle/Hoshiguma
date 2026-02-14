use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, watch::Watch};
use hoshiguma_core::accessories::extraction_airflow_sensor::types::Measurement;

pub(crate) static EXTRACTION_AIRFLOW_SENSOR_READING: Watch<
    CriticalSectionRawMutex,
    Measurement,
    1,
> = Watch::new();
