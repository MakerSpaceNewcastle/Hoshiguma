use chrono::{DateTime, TimeZone, Utc};
use core::net::{IpAddr, SocketAddr};
use defmt::{error, info};
use embassy_net::{
    Stack,
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
};
use sntpc::{NtpContext, NtpTimestampGenerator};
use sntpc_net_embassy::UdpSocketWrapper;

const NTP_SERVER: &str = "time.cloudflare.com";

pub(crate) async fn get_unix_timestamp(stack: Stack<'_>) -> Result<DateTime<Utc>, ()> {
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
    let socket = UdpSocketWrapper::new(socket);

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
                Ok(time) => match Utc.timestamp_opt(
                    time.seconds.into(),
                    sntpc::fraction_to_nanoseconds(time.seconds_fraction),
                ) {
                    chrono::offset::LocalResult::Single(time) => {
                        info!("NTP time response: {:?}", time);
                        Ok(time)
                    }
                    chrono::offset::LocalResult::Ambiguous(_, _) => {
                        error!("Error converting time: ambiguous");
                        Err(())
                    }
                    chrono::offset::LocalResult::None => {
                        error!("Error converting time");
                        Err(())
                    }
                },
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
struct TimestampGen {}

impl NtpTimestampGenerator for TimestampGen {
    fn init(&mut self) {}

    fn timestamp_sec(&self) -> u64 {
        0
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        0
    }
}
