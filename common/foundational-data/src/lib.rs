#![cfg_attr(not(feature = "std"), no_std)]

pub mod koishi;
pub mod satori;

use serde::{Deserialize, Serialize};

#[cfg(feature = "std")]
pub type String = std::string::String;
#[cfg(not(feature = "std"))]
pub type String = heapless::String<64>;

pub type TimeMillis = u32;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Message<P: Clone> {
    pub time: TimeMillis,
    pub iteration_id: Option<u32>,
    pub payload: Payload<P>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Payload<P> {
    Boot(Boot),
    Panic(Panic),
    Application(P),
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Boot {
    pub name: String,
    pub git_revision: String,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Panic {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}

impl From<&core::panic::PanicInfo<'_>> for Panic {
    fn from(info: &core::panic::PanicInfo) -> Self {
        match info.location() {
            None => Panic::default(),
            Some(loc) => Self {
                #[allow(clippy::unnecessary_fallible_conversions)]
                file: loc.file().try_into().ok(),
                line: Some(loc.line()),
                column: Some(loc.column()),
            },
        }
    }
}
