use defmt::Format;
use serde::{Deserialize, Serialize};

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HmiBacklightMode {
    AlwaysOn,
    Auto,
}
