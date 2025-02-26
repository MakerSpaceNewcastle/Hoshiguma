use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum MachineOperationLockout {
    Permitted,
    PermittedUntilIdle,
    Denied,
}
