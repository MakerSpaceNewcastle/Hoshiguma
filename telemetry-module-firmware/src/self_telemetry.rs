//! Produces data points for the telemetry module itself.
//!
//! Sent on boot:
//! - firmware Git revision
//! - boot reason
//!
//! Sent at a 1 minute interval:
//! - uptime
//! - network config/link age
//! - RPC request counters
//! - data points inserted into buffer
//! - data points discarded due to buffer overflow
//! - data submission success counter
//! - data submission failure counter

use crate::{TELEMETRY_TX, network::LINK_STATE};
use core::sync::atomic::Ordering;
use defmt::warn;
use embassy_time::{Instant, Timer};
use hoshiguma_core::telemetry::{StaticTelemetryDataPoint, TelemetryStrValue, TelemetryValue};
use portable_atomic::{AtomicU64, AtomicUsize};

pub(crate) static RPC_REQUEST_ISREADY: AtomicUsize = AtomicUsize::new(0);
pub(crate) static RPC_REQUEST_GETTIME: AtomicUsize = AtomicUsize::new(0);
pub(crate) static RPC_REQUEST_STRINGSMETADATA: AtomicUsize = AtomicUsize::new(0);
pub(crate) static RPC_REQUEST_CLEARSTRINGS: AtomicUsize = AtomicUsize::new(0);
pub(crate) static RPC_REQUEST_PUSHSTRING: AtomicUsize = AtomicUsize::new(0);
pub(crate) static RPC_REQUEST_DATAPOINT: AtomicUsize = AtomicUsize::new(0);
pub(crate) static RPC_REQUEST_DATAPOINT_FAIL_TOLINE: AtomicUsize = AtomicUsize::new(0);
pub(crate) static RPC_REQUEST_DATAPOINT_FAIL_RENDER: AtomicUsize = AtomicUsize::new(0);

pub(crate) static DATA_POINTS_ACCEPTED: AtomicUsize = AtomicUsize::new(0);
pub(crate) static DATA_POINTS_DISCARDED: AtomicUsize = AtomicUsize::new(0);

pub(crate) static TELEGRAF_SUBMIT_SUCCESS: AtomicUsize = AtomicUsize::new(0);
pub(crate) static TELEGRAF_SUBMIT_FAIL: AtomicUsize = AtomicUsize::new(0);

pub(crate) static STRING_REGISTRY_LAST_MODIFIED: AtomicU64 = AtomicU64::new(0);
pub(crate) static STRING_REGISTRY_SIZE: AtomicUsize = AtomicUsize::new(0);

pub(crate) fn string_registry_last_modification_age_ms() -> u64 {
    Instant::now()
        .as_millis()
        .saturating_sub(STRING_REGISTRY_LAST_MODIFIED.load(Ordering::Relaxed))
}

#[embassy_executor::task]
pub(crate) async fn task() {
    let metric_tx = TELEMETRY_TX.publisher().unwrap();

    macro_rules! publish_datapoint {
        ($datapoint:expr) => {
            match $datapoint.to_influx_line_string() {
                Ok(line) => {
                    metric_tx.publish_immediate(line);
                }
                Err(e) => {
                    warn!("Failed to format {}: {}", $datapoint, e);
                }
            };
        };
    }

    // Send data points that only change on boot
    {
        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_git_revision",
            field: "value",
            value: TelemetryValue::StaticString(git_version::git_version!()),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_boot_reason",
            field: "value",
            value: TelemetryValue::StaticString(crate::boot_reason().telemetry_str()),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);
    }

    // Wait a little while before sending first data points
    // FIXME: should this instead wait for the network to be up?
    Timer::after_secs(15).await;

    loop {
        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_uptime",
            field: "value",
            value: TelemetryValue::U64(Instant::now().as_millis()),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let link_state = LINK_STATE.lock(|v| v.borrow().clone());
        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_network",
            field: "config_age",
            value: TelemetryValue::U64(link_state.age().as_millis()),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_rpc,request=is_ready",
            field: "count",
            value: TelemetryValue::Usize(RPC_REQUEST_ISREADY.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_rpc,request=get_time",
            field: "count",
            value: TelemetryValue::Usize(RPC_REQUEST_GETTIME.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_rpc,request=strings_metadata",
            field: "count",
            value: TelemetryValue::Usize(RPC_REQUEST_STRINGSMETADATA.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_rpc,request=clear_strings",
            field: "count",
            value: TelemetryValue::Usize(RPC_REQUEST_CLEARSTRINGS.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_rpc,request=push_string",
            field: "count",
            value: TelemetryValue::Usize(RPC_REQUEST_PUSHSTRING.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_rpc,request=data_point",
            field: "count",
            value: TelemetryValue::Usize(RPC_REQUEST_DATAPOINT.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_rpc,request=data_point,result=fail,reason=to_line",
            field: "count",
            value: TelemetryValue::Usize(RPC_REQUEST_DATAPOINT_FAIL_TOLINE.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_rpc,request=data_point,result=fail,reason=render",
            field: "count",
            value: TelemetryValue::Usize(RPC_REQUEST_DATAPOINT_FAIL_RENDER.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_data_points,state=accepted",
            field: "count",
            value: TelemetryValue::Usize(DATA_POINTS_ACCEPTED.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_data_points,state=discarded",
            field: "count",
            value: TelemetryValue::Usize(DATA_POINTS_DISCARDED.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_data_submissions,result=success",
            field: "count",
            value: TelemetryValue::Usize(TELEGRAF_SUBMIT_SUCCESS.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_data_submissions,result=fail",
            field: "count",
            value: TelemetryValue::Usize(TELEGRAF_SUBMIT_FAIL.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_string_registry",
            field: "size",
            value: TelemetryValue::Usize(STRING_REGISTRY_SIZE.load(Ordering::Relaxed)),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        let dp = StaticTelemetryDataPoint {
            measurement: "telemetry_module_string_registry",
            field: "last_modify_age",
            value: TelemetryValue::U64(string_registry_last_modification_age_ms()),
            timestamp_nanoseconds: None,
        };
        publish_datapoint!(dp);

        // Send data points (approximately) every minute
        Timer::after_secs(60).await;
    }
}
