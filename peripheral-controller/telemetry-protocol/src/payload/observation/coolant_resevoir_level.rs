use serde::{Deserialize, Serialize};

pub type CoolantResevoirLevelReading = Result<CoolantResevoirLevel, ()>;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum CoolantResevoirLevel {
    Full,
    Low,
    Empty,
}
