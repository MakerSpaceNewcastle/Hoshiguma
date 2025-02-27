use crate::TelemString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Info {
    pub git_revision: TelemString,
}
