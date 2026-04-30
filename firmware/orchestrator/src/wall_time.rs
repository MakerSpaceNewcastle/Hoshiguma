use chrono::{DateTime, Utc};
use embassy_net::Stack;

async fn get_time_from_telemetry_module(stack: Stack<'static>) -> Result<DateTime<Utc>, ()> {
    todo!()
}
