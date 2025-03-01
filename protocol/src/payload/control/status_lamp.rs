use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct StatusLamp {
    pub red: bool,
    pub amber: bool,
    pub green: bool,
}
