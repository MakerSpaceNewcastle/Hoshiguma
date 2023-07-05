use const_format::formatcp;

const TOPIC_PREFIX: &'static str = "doot/testo";

pub const TOPIC_ALIVE: &'static str = formatcp!("{}/alive", TOPIC_PREFIX);
pub const ALIVE_PAYLOAD_ONLINE: &'static [u8] = b"online";
pub const ALIVE_PAYLOAD_OFFLINE: &'static [u8] = b"offline";

pub const TOPIC_STATUS: &'static str = formatcp!("{}/status", TOPIC_PREFIX);
