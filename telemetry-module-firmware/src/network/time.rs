use core::{
    net::{IpAddr, SocketAddr},
    sync::atomic::Ordering,
    time::Duration,
};
use defmt::{error, info};
use embassy_net::{
    dns::DnsQueryType,
    udp::{PacketMetadata, UdpSocket},
    Stack,
};
use embassy_time::{Instant, Timer};
use portable_atomic::{AtomicI128, AtomicU64};
use sntpc::{NtpContext, NtpTimestampGenerator};

static US_SINCE_UNIX_EPOCH: AtomicI128 = AtomicI128::new(0);
static US_SINCE_BOOT: AtomicU64 = AtomicU64::new(0);

#[embassy_executor::task]
pub(super) async fn task(stack: Stack<'static>) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("network time sync").await;

    loop {
        time_sync(stack).await;

        if wall_time().is_some() {
            Timer::after_secs(120).await;
        } else {
            Timer::after_secs(5).await;
        }
    }
}

const NTP_SERVER: &str = "pool.ntp.org";

async fn time_sync(stack: Stack<'_>) {
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
                return;
            }

            let addr: IpAddr = ntp_addrs[0].into();
            let result = sntpc::get_time(SocketAddr::from((addr, 123)), &socket, context).await;

            match result {
                Ok(time) => {
                    info!("{:?}", time);

                    let now = Instant::now().as_micros();
                    US_SINCE_UNIX_EPOCH.add(time.offset.into(), Ordering::Relaxed);
                    US_SINCE_BOOT.store(now, Ordering::Relaxed);

                    info!("Wall time: {:?}", wall_time());
                }
                Err(e) => {
                    error!("Error getting time: {:?}", e);
                }
            }
        }
        Err(_) => error!("Failed to resolve DNS"),
    }
}

pub(crate) fn wall_time() -> Option<Duration> {
    let since_epoch = US_SINCE_UNIX_EPOCH.load(Ordering::Relaxed) as u64;
    match since_epoch {
        0 => None,
        _ => Some(Duration::from_micros(since_epoch) + time_sync_age()),
    }
}

pub(crate) fn time_sync_age() -> Duration {
    let since_boot = US_SINCE_BOOT.load(Ordering::Relaxed);
    let us_to_add = Instant::now().as_micros() - since_boot;
    Duration::from_micros(us_to_add)
}

#[derive(Copy, Clone, Default)]
struct TimestampGen {
    wall_time: Duration,
}

impl NtpTimestampGenerator for TimestampGen {
    fn init(&mut self) {
        self.wall_time = wall_time().unwrap_or(Duration::ZERO);
    }

    fn timestamp_sec(&self) -> u64 {
        self.wall_time.as_secs()
    }

    fn timestamp_subsec_micros(&self) -> u32 {
        self.wall_time.subsec_micros()
    }
}
