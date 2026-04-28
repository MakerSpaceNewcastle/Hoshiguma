use core::net::Ipv4Addr;

pub const TELEMETRY_MODULE_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 1);
pub const ORCHESTRATOR_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 2);
pub const RUIDA_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 3);
pub const HMI_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 4);
pub const COOLER_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 5);
pub const REAR_SENSOR_BOARD_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 6);

pub const CONTROL_PORT: u16 = 2000;
pub const NOTIFICATION_PORT: u16 = 2001;
