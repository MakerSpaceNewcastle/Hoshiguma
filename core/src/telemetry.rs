use crate::string_registry::StringRegistry;
use core::fmt::{Display, Write};
use heapless::{String, Vec};
use serde::{Deserialize, Serialize};

pub trait AsTelemetry<const STRINGS: usize, const MAX_ITEMS: usize> {
    fn strings() -> [&'static str; STRINGS];
    fn telemetry(&self) -> Vec<StaticTelemetryDataPoint, MAX_ITEMS>;
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
    pub timestamp_nanoseconds: Option<u64>,
}

pub type StaticTelemetryDataPoint = TelemetryDataPoint<&'static str>;

impl StaticTelemetryDataPoint {
    pub fn to_templated_data_point<
        const STRING_REGISTRY_CAPACITY: usize,
        const REGISTERED_STRING_CAPACITY: usize,
    >(
        self,
        sr: &StringRegistry<STRING_REGISTRY_CAPACITY, REGISTERED_STRING_CAPACITY>,
    ) -> Result<TemplatedTelemetryDataPoint> {
        let measurement = sr
            .get_index(self.measurement)
            .ok_or(Error::MissingString(self.measurement))?;

        let field = sr
            .get_index(self.field)
            .ok_or(Error::MissingString(self.field))?;

        Ok(TemplatedTelemetryDataPoint {
            measurement,
            field,
            value: match self.value {
                TelemetryValue::Usize(value) => Ok(TelemetryValue::Usize(value)),
                TelemetryValue::U64(value) => Ok(TelemetryValue::U64(value)),
                TelemetryValue::Float32(value) => Ok(TelemetryValue::Float32(value)),
                TelemetryValue::Float64(value) => Ok(TelemetryValue::Float64(value)),
                TelemetryValue::Bool(value) => Ok(TelemetryValue::Bool(value)),
                TelemetryValue::StaticString(value) => Ok(TelemetryValue::StaticString(
                    sr.get_index(value).ok_or(Error::MissingString(value))?,
                )),
                TelemetryValue::DynamicString(value) => Ok(TelemetryValue::DynamicString(value)),
            }?,
            timestamp_nanoseconds: self.timestamp_nanoseconds,
        })
    }

    pub fn to_influx_line_string<const INFLUX_LINE_CAPACITY: usize>(
        &self,
    ) -> Result<String<INFLUX_LINE_CAPACITY>> {
        match &self.value {
            TelemetryValue::Usize(value) => format_influx_line(
                self.measurement,
                self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::U64(value) => format_influx_line(
                self.measurement,
                self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::Float32(value) => format_influx_line(
                self.measurement,
                self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::Float64(value) => format_influx_line(
                self.measurement,
                self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::Bool(value) => format_influx_line_str(
                self.measurement,
                self.field,
                match value {
                    true => "true",
                    false => "false",
                },
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::StaticString(value) => format_influx_line_str(
                self.measurement,
                self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::DynamicString(value) => format_influx_line_str(
                self.measurement,
                self.field,
                value,
                self.timestamp_nanoseconds,
            ),
        }
    }
}

pub type TemplatedTelemetryDataPoint = TelemetryDataPoint<usize>;

impl TemplatedTelemetryDataPoint {
    pub fn to_rendered_data_point<
        const STRING_REGISTRY_CAPACITY: usize,
        const REGISTERED_STRING_CAPACITY: usize,
    >(
        self,
        sr: &StringRegistry<STRING_REGISTRY_CAPACITY, REGISTERED_STRING_CAPACITY>,
    ) -> Result<RenderedTelemetryDataPoint<REGISTERED_STRING_CAPACITY>> {
        let measurement = sr
            .get_string(self.measurement)
            .ok_or(Error::NoStringAtIndex(self.measurement))?;

        let field = sr
            .get_string(self.field)
            .ok_or(Error::NoStringAtIndex(self.field))?;

        Ok(RenderedTelemetryDataPoint {
            measurement,
            field,
            value: match self.value {
                TelemetryValue::Usize(value) => Ok(TelemetryValue::Usize(value)),
                TelemetryValue::U64(value) => Ok(TelemetryValue::U64(value)),
                TelemetryValue::Float32(value) => Ok(TelemetryValue::Float32(value)),
                TelemetryValue::Float64(value) => Ok(TelemetryValue::Float64(value)),
                TelemetryValue::Bool(value) => Ok(TelemetryValue::Bool(value)),
                TelemetryValue::StaticString(idx) => Ok(TelemetryValue::StaticString(
                    sr.get_string(idx).ok_or(Error::NoStringAtIndex(idx))?,
                )),
                TelemetryValue::DynamicString(value) => Ok(TelemetryValue::DynamicString(value)),
            }?,
            timestamp_nanoseconds: self.timestamp_nanoseconds,
        })
    }
}

pub type RenderedTelemetryDataPoint<const REGISTERED_STRING_CAPACITY: usize> =
    TelemetryDataPoint<String<REGISTERED_STRING_CAPACITY>>;

impl<const SL: usize> RenderedTelemetryDataPoint<SL> {
    pub fn to_influx_line_string<const INFLUX_LINE_CAPACITY: usize>(
        self,
    ) -> Result<String<INFLUX_LINE_CAPACITY>> {
        match &self.value {
            TelemetryValue::Usize(value) => format_influx_line(
                &self.measurement,
                &self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::U64(value) => format_influx_line(
                &self.measurement,
                &self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::Float32(value) => format_influx_line(
                &self.measurement,
                &self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::Float64(value) => format_influx_line(
                &self.measurement,
                &self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::Bool(value) => format_influx_line_str(
                &self.measurement,
                &self.field,
                match value {
                    true => "true",
                    false => "false",
                },
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::StaticString(value) => format_influx_line_str(
                &self.measurement,
                &self.field,
                value,
                self.timestamp_nanoseconds,
            ),
            TelemetryValue::DynamicString(value) => format_influx_line_str(
                &self.measurement,
                &self.field,
                value,
                self.timestamp_nanoseconds,
            ),
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
    StaticString(StringType),
    DynamicString(String<24>),
}

fn format_influx_timestamp(timestamp_ns: Option<u64>) -> Result<String<20>> {
    let mut timestamp = String::new();

    if let Some(timestamp_ns) = timestamp_ns {
        timestamp
            .write_fmt(format_args!(" {timestamp_ns}"))
            .map_err(|_| Error::FormatError)?;
    }

    Ok(timestamp)
}

fn format_influx_line<const LEN: usize, T: Display>(
    measurement: &str,
    field: &str,
    value: T,
    timestamp_ns: Option<u64>,
) -> Result<String<LEN>> {
    let timestamp = format_influx_timestamp(timestamp_ns)?;

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
    timestamp_ns: Option<u64>,
) -> Result<String<LEN>> {
    let timestamp = format_influx_timestamp(timestamp_ns)?;

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

    #[test]
    fn to_influx_line_usize_no_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::Usize(42),
            timestamp_nanoseconds: None,
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
            timestamp_nanoseconds: None,
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
            timestamp_nanoseconds: None,
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
            timestamp_nanoseconds: None,
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

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::Usize(42),
            timestamp_nanoseconds: Some(1770452242881786000),
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<48>()
            .unwrap();

        assert_eq!(s, "zero one=42 1770452242881786000");
    }

    #[test]
    fn to_influx_line_f32_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::Float32(3.14),
            timestamp_nanoseconds: Some(1770452242881786000),
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<48>()
            .unwrap();

        assert_eq!(s, "zero one=3.14 1770452242881786000");
    }

    #[test]
    fn to_influx_line_reg_str_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one", "two"]).unwrap();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::StaticString(2),
            timestamp_nanoseconds: Some(1770452242881786000),
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<48>()
            .unwrap();

        assert_eq!(s, "zero one=\"two\" 1770452242881786000");
    }

    #[test]
    fn to_influx_line_str_timestamp() {
        let sr = StringRegistry::<8, 32>::from_slice(&["zero", "one"]).unwrap();

        let p = TelemetryDataPoint::<usize> {
            measurement: 0,
            field: 1,
            value: TelemetryValue::DynamicString("nope".try_into().unwrap()),
            timestamp_nanoseconds: Some(1770452242881786000),
        };

        let s = p
            .to_rendered_data_point(&sr)
            .unwrap()
            .to_influx_line_string::<48>()
            .unwrap();

        assert_eq!(s, "zero one=\"nope\" 1770452242881786000");
    }
}
