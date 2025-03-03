use embassy_net::{IpAddress, Ipv4Address};

pub(crate) const MQTT_BROKER_IP: IpAddress = IpAddress::Ipv4(Ipv4Address::new(192, 168, 8, 183));
pub(super) const MQTT_BROKER_PORT: u16 = 1883;

pub(super) const MQTT_CLIENT_ID: &str = "hoshiguma-telemetry-module";
pub(super) const MQTT_USERNAME: &str = "hoshiguma";

pub(super) mod topics {
    use const_format::formatcp;

    const ROOT: &str = "hosthiguma";

    const TELEMETRY_MODULE: &str = formatcp!("{ROOT}/telemetry-module");

    pub(crate) const TELEMETRY_MODULE_ONLINE: &str = formatcp!("{TELEMETRY_MODULE}/online");
    pub(crate) const TELEMETRY_MODULE_VERSION: &str = formatcp!("{TELEMETRY_MODULE}/version");

    pub(crate) const TELEMETRY_EVENTS: &str = formatcp!("{ROOT}/events");
}
