use serde::{Deserialize, Serialize};

pub type CoolantResevoirLevelReading = Result<CoolantResevoirLevel, ()>;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum CoolantResevoirLevel {
    Full,
    Low,
    Empty,
}
