pub mod koishi;
pub mod satori;

use serde::{Deserialize, Serialize};

pub type TimeMillis = u32;

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Message<P> {
    pub time: TimeMillis,
    pub iteration_id: Option<u32>,
    pub payload: Payload<P>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Payload<P> {
    Boot(Boot),
    Panic(Panic),
    Application(P),
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Boot {
    pub name: String,
    pub git_revision: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[allow(dead_code)]
pub struct Panic {
    pub file: Option<String>,
    pub line: Option<u32>,
    pub column: Option<u32>,
}
