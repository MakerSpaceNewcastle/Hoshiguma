use chrono::{DateTime, Utc};
use core::fmt::{Display, Write};
use heapless::String;

pub type FormatInfluxResult<const LEN: usize> = Result<String<LEN>, core::fmt::Error>;

pub fn format_influx_line<const LEN: usize, T: Display>(
    measurement: &str,
    field: &str,
    value: T,
    timestamp: Option<DateTime<Utc>>,
) -> FormatInfluxResult<LEN> {
    let timestamp = format_influx_timestamp(timestamp)?;

    let mut line_str = String::new();

    line_str.write_fmt(format_args!("{measurement} {field}={value}{timestamp}"))?;

    Ok(line_str)
}

pub fn format_influx_line_str<const LEN: usize, T: Display>(
    measurement: &str,
    field: &str,
    value: T,
    timestamp: Option<DateTime<Utc>>,
) -> FormatInfluxResult<LEN> {
    let timestamp = format_influx_timestamp(timestamp)?;

    let mut line_str = String::new();

    line_str.write_fmt(format_args!("{measurement} {field}=\"{value}\"{timestamp}"))?;

    Ok(line_str)
}

fn format_influx_timestamp(timestamp: Option<DateTime<Utc>>) -> FormatInfluxResult<20> {
    let mut s = String::new();

    if let Some(timestamp) = timestamp {
        let timestamp_ns = timestamp.timestamp_nanos_opt().unwrap();

        s.write_fmt(format_args!(" {timestamp_ns}"))?;
    }

    Ok(s)
}
