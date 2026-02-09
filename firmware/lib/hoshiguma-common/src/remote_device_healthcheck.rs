use crate::{
    changed::ObservedValue,
    network::send_request,
    telemetry::{FormatInfluxResult, format_influx_line},
};
use core::{fmt::Write, marker::PhantomData, net::Ipv4Addr};
use defmt::{info, warn};
use embassy_net::Stack;
use embassy_time::{Duration, Instant};
use heapless::String;
use hoshiguma_api::{
    API_PORT, BootReason, ExpectedResponse, GitRevisionString, MessagePayload, Severity,
    SystemInformation, telemetry_bridge::TELEMETRY_DATA_POINT_MAX_LEN,
};
use serde::{Serialize, de::DeserializeOwned};

type TelemetryMeasurementName = String<64>;

pub struct RemoteDeviceHealthCheck<ReqT, RespT, SeverityFn, TelemFn>
where
    SeverityFn: AsyncFnMut(Severity) -> (),
{
    net_stack: Stack<'static>,
    device_ip: Ipv4Addr,

    last_contact: Instant,
    severity: ObservedValue<Severity>,
    on_severity_changed: SeverityFn,

    telemetry_name_git_revision: TelemetryMeasurementName,
    telemetry_name_boot_reason: TelemetryMeasurementName,
    telemetry_name_uptime: TelemetryMeasurementName,
    telemetry_name_up: TelemetryMeasurementName,
    git_revision: ObservedValue<GitRevisionString>,
    boot_reason: ObservedValue<BootReason>,
    on_telemetry: TelemFn,

    _api_types: PhantomData<(ReqT, RespT)>,
}

impl<ReqT, RespT, SeverityFn, TelemFn> RemoteDeviceHealthCheck<ReqT, RespT, SeverityFn, TelemFn>
where
    ReqT: ExpectedResponse<Response = RespT> + Default + MessagePayload + Serialize,
    RespT: Into<SystemInformation> + MessagePayload + DeserializeOwned,
    SeverityFn: AsyncFnMut(Severity) -> (),
    TelemFn: Fn(FormatInfluxResult<TELEMETRY_DATA_POINT_MAX_LEN>),
{
    pub fn new(
        net_stack: Stack<'static>,
        device_name: &str,
        device_ip: Ipv4Addr,
        on_severity_changed: SeverityFn,
        on_telemetry: TelemFn,
    ) -> Self {
        let mut telemetry_name_git_revision = String::new();
        telemetry_name_git_revision
            .write_fmt(format_args!("{device_name}_git_revision"))
            .unwrap();

        let mut telemetry_name_boot_reason = String::new();
        telemetry_name_boot_reason
            .write_fmt(format_args!("{device_name}_boot_reason"))
            .unwrap();

        let mut telemetry_name_uptime = String::new();
        telemetry_name_uptime
            .write_fmt(format_args!("{device_name}_uptime"))
            .unwrap();

        let mut telemetry_name_up = String::new();
        telemetry_name_up
            .write_fmt(format_args!("{device_name}_up"))
            .unwrap();

        Self {
            net_stack,
            device_ip,
            last_contact: Instant::MIN,
            severity: ObservedValue::default(),
            on_severity_changed,
            telemetry_name_git_revision,
            telemetry_name_boot_reason,
            telemetry_name_uptime,
            telemetry_name_up,
            git_revision: ObservedValue::default(),
            boot_reason: ObservedValue::default(),
            on_telemetry,
            _api_types: PhantomData,
        }
    }

    pub async fn check(&mut self) {
        /// The amount of time the device can be unreachable before it is considered to have failed.
        const TIME_BEFORE_FAILED: Duration = Duration::from_secs(10);

        let new_severity = match self.get_device_system_information().await {
            Ok(info) => {
                self.last_contact = Instant::now();

                // Send telemetry: Git revision
                self.git_revision
                    .update_and(info.git_revision, |git_revision| {
                        (self.on_telemetry)(format_influx_line(
                            format_args!(
                                "{} value=\"{git_revision}\"",
                                &self.telemetry_name_git_revision
                            ),
                            None,
                        ));
                    });

                // Send telemetry: boot reason
                self.boot_reason
                    .update_and(info.boot_reason, |boot_reason| {
                        (self.on_telemetry)(format_influx_line(
                            format_args!(
                                "{} value=\"{boot_reason}\"",
                                &self.telemetry_name_boot_reason
                            ),
                            None,
                        ));
                    });

                // Send telemetry: uptime
                (self.on_telemetry)(format_influx_line(
                    format_args!(
                        "{} value={}",
                        &self.telemetry_name_uptime,
                        info.uptime.as_millis(),
                    ),
                    None,
                ));

                Severity::Normal
            }
            Err(()) => {
                let since_last_contact = Instant::now() - self.last_contact;
                warn!(
                    "{}ms since last contact with device",
                    since_last_contact.as_millis()
                );

                if since_last_contact > TIME_BEFORE_FAILED {
                    Severity::Critical
                } else {
                    Severity::Warning
                }
            }
        };

        // Send telemetry: up
        (self.on_telemetry)(format_influx_line(
            format_args!(
                "{} value={}",
                &self.telemetry_name_up,
                match new_severity {
                    Severity::Normal => 1,
                    _ => 0,
                },
            ),
            None,
        ));

        // Check for severity change and notify
        self.severity
            .update_and_async(new_severity, async |severity| {
                info!("Device is now {}", severity);
                (self.on_severity_changed)(severity).await;
            })
            .await;
    }

    async fn get_device_system_information(&self) -> Result<SystemInformation, ()> {
        let response: RespT = send_request(
            self.net_stack,
            self.device_ip,
            API_PORT,
            1,
            &ReqT::default(),
        )
        .await
        .map_err(|e| {
            warn!("Failed to query system information: {}", e);
        })?;

        Ok(response.into())
    }
}
