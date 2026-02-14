use super::types::Measurement;
use crate::types::{BootReason, GitRevisionString};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    GetUptime,
    GetBootReason,
    GetGitRevision,

    GetMeasurement,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    GetUptime(u64),
    GetBootReason(BootReason),
    GetGitRevision(GitRevisionString),

    GetMeasurement(Measurement),
}
