//! Control network configuration.

use core::net::Ipv4Addr;

pub const TELEMETRY_MODULE_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 1);
pub const TELEMETRY_MODULE_MAC_ADDRESS: [u8; 6] = [0xF2, 0xFA, 0xEC, 0x8D, 0x87, 0x01];

pub const TELEMETRY_MODULE_MAC_ADDRESS_PUBLIC: [u8; 6] = [0xF2, 0xFA, 0xEC, 0x8D, 0x88, 0x01];

pub const ORCHESTRATOR_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 2);
pub const ORCHESTRATOR_MAC_ADDRESS: [u8; 6] = [0xF2, 0xFA, 0xEC, 0x8D, 0x88, 0x02];

pub const RUIDA_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 3);

pub const HMI_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 4);
pub const HMI_MAC_ADDRESS: [u8; 6] = [0xF2, 0xFA, 0xEC, 0x8D, 0x88, 0x04];

pub const COOLER_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 5);
pub const COOLER_MAC_ADDRESS: [u8; 6] = [0xF2, 0xFA, 0xEC, 0x8D, 0x88, 0x05];

pub const REAR_SENSOR_BOARD_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 6);
pub const REAR_SENSOR_BOARD_MAC_ADDRESS: [u8; 6] = [0xF2, 0xFA, 0xEC, 0x8D, 0x88, 0x06];

pub const AUX_CONTROL_PORT: u16 = 2001;
