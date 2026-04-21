use core::net::Ipv4Addr;

pub const COOLER_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 4);
pub const COOLER_MAC_ADDRESS: [u8; 6] = [0x02, 0x00, 0x00, 0xff, 0x22, 0x22];

pub const REAR_SENSOR_BOARD_IP_ADDRESS: Ipv4Addr = Ipv4Addr::new(10, 69, 69, 5);
pub const REAR_SENSOR_BOARD_MAC_ADDRESS: [u8; 6] = [0x02, 0x00, 0x00, 0xff, 0x22, 0x23];

pub const AUX_CONTROL_PORT: u16 = 2001;
