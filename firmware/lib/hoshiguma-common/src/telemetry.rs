use chrono::{DateTime, Utc};
use core::fmt::{Display, Write};
use heapless::String;
use serde::{Deserialize, Serialize};

pub fn format_influx_timestamp(timestamp: Option<DateTime<Utc>>) -> Result<String<20>> {
    let mut s = String::new();

    if let Some(timestamp) = timestamp {
        let timestamp_ns = timestamp.timestamp_nanos_opt().unwrap();

        s.write_fmt(format_args!(" {timestamp_ns}"))
            .map_err(|_| Error::FormatError)?;
    }

    Ok(s)
}

pub fn format_influx_line<const LEN: usize, T: Display>(
    measurement: &str,
    field: &str,
    value: T,
    timestamp: Option<DateTime<Utc>>,
) -> Result<String<LEN>> {
    let timestamp = format_influx_timestamp(timestamp)?;

    let mut line_str = String::new();

    line_str
        .write_fmt(format_args!("{measurement} {field}={value}{timestamp}"))
        .map_err(|_| Error::FormatError)?;

    Ok(line_str)
}

pub fn format_influx_line_str<const LEN: usize, T: Display>(
    measurement: &str,
    field: &str,
    value: T,
    timestamp: Option<DateTime<Utc>>,
) -> Result<String<LEN>> {
    let timestamp = format_influx_timestamp(timestamp)?;

    let mut line_str = String::new();

    line_str
        .write_fmt(format_args!("{measurement} {field}=\"{value}\"{timestamp}"))
        .map_err(|_| Error::FormatError)?;

    Ok(line_str)
}

pub type Result<T> = core::result::Result<T, Error>;

#[derive(Debug, defmt::Format, Clone, PartialEq, Serialize, Deserialize)]
pub enum Error {
    FormatError,
    MissingString(&'static str),
    NoStringAtIndex(usize),
}
