use core::{
    net::{IpAddr, SocketAddr},
    time::Duration,
};
use defmt::{error, info};
use embassy_net::{
    Stack,
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use sntpc::{NtpContext, NtpTimestampGenerator};

const NTP_SERVER: &str = "time.cloudflare.com";

pub(crate) async fn get_unix_timestamp_offset(stack: Stack<'_>) -> Result<i64, ()> {
    info!("Syncing time now");

    // Create UDP socket
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];

    let mut socket = UdpSocket::new(
        stack,
        &mut rx_meta,
        &mut rx_buffer,
        &mut tx_meta,
        &mut tx_buffer,
    );
    socket.bind(123).unwrap();

    let context = NtpContext::new(TimestampGen::default());

    match stack.dns_query(NTP_SERVER, DnsQueryType::A).await {
        Ok(ntp_addrs) => {
            if ntp_addrs.is_empty() {
                error!("Failed to resolve DNS");
                return Err(());
            }

            let addr: IpAddr = ntp_addrs[0].into();
            let result = sntpc::get_time(SocketAddr::from((addr, 123)), &socket, context).await;

            match result {
                Ok(time) => {
                    info!("NTP time response: {:?}", time);
                    Ok(time.offset)
                }
                Err(e) => {
                    error!("Error getting time: {:?}", e);
                    Err(())
                }
            }
        }
        Err(_) => {
            error!("Failed to resolve DNS");
            Err(())
        }
    }
}

#[derive(Copy, Clone, Default)]
struct TimestampGen {
    wall_time: Duration,
}

impl NtpTimestampGenerator for TimestampGen {
    fn init(&mut self) {
        self.wall_time = Duration::ZERO;
    }

    fn timestamp_sec(&self) -> u64 {
        self.wall_time.as_secs()
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        self.wall_time.subsec_micros()
    }
}
