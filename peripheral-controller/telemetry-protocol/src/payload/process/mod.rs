mod alarms;
mod lockout;
mod monitor;

pub use self::{
    alarms::ActiveAlarms,
    lockout::MachineOperationLockout,
    monitor::{Monitor, MonitorState, MonitorStatus},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum ProcessPayload {
    Monitor(MonitorStatus),
    Alarms(ActiveAlarms),
    Lockout(MachineOperationLockout),
}