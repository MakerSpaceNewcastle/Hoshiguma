use super::types::{CompressorState, CoolantPumpState, RadiatorFanState, State};
use crate::types::{BootReason, GitRevisionString};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Request {
    GetUptime,
    GetBootReason,
    GetGitRevision,

    GetState,

    SetRadiatorFan(RadiatorFanState),
    SetCompressor(CompressorState),
    SetCoolantPump(CoolantPumpState),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Response {
    GetUptime(u64),
    GetBootReason(BootReason),
    GetGitRevision(GitRevisionString),

    GetState(State),

    SetRadiatorFan,
    SetCompressor,
    SetCoolantPump,
}
