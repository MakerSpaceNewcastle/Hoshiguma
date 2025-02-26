use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MachineRunStatus {
    Idle,
    Running,
}
