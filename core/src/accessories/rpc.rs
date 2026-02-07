use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    Cooler(super::cooler::rpc::Request),
    ExtractionAirflowSensor(super::extraction_airflow_sensor::rpc::Request),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    Cooler(super::cooler::rpc::Response),
    ExtractionAirflowSensor(super::extraction_airflow_sensor::rpc::Response),
}
