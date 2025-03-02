use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum MachinePower {
    On,
    Off,
}
