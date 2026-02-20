use chrono::{DateTime, Utc, serde::ts_nanoseconds_option};
use core::fmt::{Display, Write};
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

pub trait AsTelemetry<StringType, const MAX_ITEMS: usize> {
    fn telemetry(&self) -> Vec<TelemetryDataPoint<StringType>, MAX_ITEMS>;
}

pub trait TelemetryStrValue {
    fn telemetry_str(&self) -> &'static str;
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub struct TelemetryDataPoint<StringType> {
    pub measurement: StringType,
    pub field: StringType,
    pub value: TelemetryValue<StringType>,
    #[serde(with = "ts_nanoseconds_option")]
    pub timestamp: Option<DateTime<Utc>>,
}

impl<StringType> TelemetryDataPoint<StringType> {
    pub fn to_influx_line_string<const INFLUX_LINE_CAPACITY: usize>(
        &self,
    ) -> Result<String<INFLUX_LINE_CAPACITY>> {
        match &self.value {
            TelemetryValue::Usize(value) => {
                format_influx_line(self.measurement, self.field, value, self.timestamp)
            }
            TelemetryValue::U64(value) => {
                format_influx_line(self.measurement, self.field, value, self.timestamp)
            }
            TelemetryValue::Float32(value) => {
                format_influx_line(self.measurement, self.field, value, self.timestamp)
            }
            TelemetryValue::Float64(value) => {
                format_influx_line(self.measurement, self.field, value, self.timestamp)
            }
            TelemetryValue::Bool(value) => format_influx_line_str(
                self.measurement,
                self.field,
                match value {
                    true => "true",
                    false => "false",
                },
                self.timestamp,
            ),
            TelemetryValue::String(value) => {
                format_influx_line_str(self.measurement, self.field, value, self.timestamp)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum TelemetryValue<StringType> {
    Usize(usize),
    U64(u64),
    Float32(f32),
    Float64(f64),
    Bool(bool),
    String(StringType),
}

fn format_influx_timestamp(timestamp: Option<DateTime<Utc>>) -> Result<String<20>> {
    let mut s = String::new();

    if let Some(timestamp) = timestamp {
        let timestamp_ns = timestamp.timestamp_nanos_opt().unwrap();

        s.write_fmt(format_args!(" {timestamp_ns}"))
            .map_err(|_| Error::FormatError)?;
    }

    Ok(s)
}

fn format_influx_line<const LEN: usize, T: Display>(
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

fn format_influx_line_str<const LEN: usize>(
    measurement: &str,
    field: &str,
    value: &str,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(feature = "no-std", derive(defmt::Format))]
pub enum Error {
    FormatError,
    MissingString(&'static str),
    NoStringAtIndex(usize),
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn to_influx_line_usize_no_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::Usize(42),
            timestamp: None,
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<32>()
            .unwrap();

        assert_eq!(s, "zero one=42");
    }

    #[test]
    fn to_influx_line_f32_no_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::Float32(3.14),
            timestamp: None,
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<32>()
            .unwrap();

        assert_eq!(s, "zero one=3.14");
    }

    #[test]
    fn to_influx_line_reg_str_no_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one", "two"]).unwrap();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::StaticString(2),
            timestamp: None,
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<32>()
            .unwrap();

        assert_eq!(s, "zero one=\"two\"");
    }

    #[test]
    fn to_influx_line_str_no_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::DynamicString("nope".try_into().unwrap()),
            timestamp: None,
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<32>()
            .unwrap();

        assert_eq!(s, "zero one=\"nope\"");
    }

    #[test]
    fn to_influx_line_usize_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let t = NaiveDate::from_ymd_opt(2026, 2, 7)
            .unwrap()
            .and_hms_opt(11, 39, 40)
            .unwrap()
            .and_utc();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::Usize(42),
            timestamp: Some(t),
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<48>()
            .unwrap();

        assert_eq!(s, "zero one=42 1770464380000000000");
    }

    #[test]
    fn to_influx_line_f32_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let t = NaiveDate::from_ymd_opt(2026, 2, 7)
            .unwrap()
            .and_hms_opt(11, 39, 40)
            .unwrap()
            .and_utc();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::Float32(3.14),
            timestamp: Some(t),
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<48>()
            .unwrap();

        assert_eq!(s, "zero one=3.14 1770464380000000000");
    }

    #[test]
    fn to_influx_line_reg_str_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one", "two"]).unwrap();

        let t = NaiveDate::from_ymd_opt(2026, 2, 7)
            .unwrap()
            .and_hms_opt(11, 39, 40)
            .unwrap()
            .and_utc();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::StaticString(2),
            timestamp: Some(t),
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<48>()
            .unwrap();

        assert_eq!(s, "zero one=\"two\" 1770464380000000000");
    }

    #[test]
    fn to_influx_line_str_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let t = NaiveDate::from_ymd_opt(2026, 2, 7)
            .unwrap()
            .and_hms_opt(11, 39, 40)
            .unwrap()
            .and_utc();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::DynamicString("nope".try_into().unwrap()),
            timestamp: Some(t),
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<48>()
            .unwrap();

        assert_eq!(s, "zero one=\"nope\" 1770464380000000000");
    }
}
