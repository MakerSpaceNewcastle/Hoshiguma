use chrono::{DateTime, Utc};
use core::fmt::{Arguments, Write};
use heapless::String;

pub type FormatInfluxResult<const LEN: usize> = Result<String<LEN>, core::fmt::Error>;

pub fn format_influx_line<const LEN: usize>(
    args: Arguments,
    timestamp: Option<DateTime<Utc>>,
) -> FormatInfluxResult<LEN> {
    let mut line_str = String::new();

    line_str.write_fmt(args)?;

    if let Some(timestamp) = timestamp {
        let timestamp_ns = timestamp.timestamp_nanos_opt().unwrap();

        line_str.write_fmt(format_args!(" {timestamp_ns}"))?;
    }

    Ok(line_str)
}
