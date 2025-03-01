use crate::TelemString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
#[allow(dead_code)]
pub struct Info {
    pub git_revision: TelemString,
}
