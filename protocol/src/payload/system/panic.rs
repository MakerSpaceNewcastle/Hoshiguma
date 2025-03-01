use crate::TelemString;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[allow(dead_code)]
pub struct Panic {
    pub file: Option<TelemString>,
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
