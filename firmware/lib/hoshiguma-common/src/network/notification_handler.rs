use crate::network::config::NOTIFICATION_PORT;
use core::net::Ipv4Addr;
use defmt::{Format, info, warn};
use embassy_futures::select::Either;
use embassy_net::{Stack, tcp::TcpSocket};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    channel::Receiver,
    pubsub::{PubSubChannel, Publisher, Subscriber, WaitResult},
};
use embedded_io_async::Write;
use heapless::Vec;
use hoshiguma_api::{Message, MessagePayload};
use serde::Serialize;

#[derive(Debug, Format, Clone, Copy, PartialEq, Eq)]
pub struct Subscription {
    pub ip: Ipv4Addr,
}

pub type NotificationSubscriptionChannel<const NUM_LISTENERS: usize, const NUM_NOTIFIERS: usize> =
    PubSubChannel<CriticalSectionRawMutex, Subscription, 1, NUM_NOTIFIERS, NUM_LISTENERS>;

pub type NotificationSubscriptionChannelPublisher<
    const NUM_LISTENERS: usize,
    const NUM_NOTIFIERS: usize,
> = Publisher<'static, CriticalSectionRawMutex, Subscription, 1, NUM_NOTIFIERS, NUM_LISTENERS>;

pub type NotificationSubscriptionChannelSubscriber<
    const NUM_LISTENERS: usize,
    const NUM_NOTIFIERS: usize,
> = Subscriber<'static, CriticalSectionRawMutex, Subscription, 1, NUM_NOTIFIERS, NUM_LISTENERS>;

pub async fn notification_tx_loop<
    const NUM_LISTENERS: usize,
    const NUM_NOTIFIERS: usize,
    T: MessagePayload + Serialize,
    const CAP: usize,
>(
    stack: Stack<'static>,
    id: u8,
    mut subscription_rx: NotificationSubscriptionChannelSubscriber<NUM_LISTENERS, NUM_NOTIFIERS>,
    notification_rx: Receiver<'static, CriticalSectionRawMutex, T, CAP>,
) -> ! {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut subscriptions = Vec::<Subscription, 4>::new();

    loop {
        match embassy_futures::select::select(
            subscription_rx.next_message(),
            notification_rx.receive(),
        )
        .await
        {
            Either::First(WaitResult::Lagged(n)) => {
                panic!("socket {}: subscription channel lagged by {}", id, n);
            }
            Either::First(WaitResult::Message(sub)) => {
                // Make sure the same subscription doesn't already exist
                if subscriptions.iter().any(|s| s.ip == sub.ip) {
                    info!(
                        "socket {}: subscription {:?} already exists, ignoring",
                        id, sub.ip
                    );
                }

                if subscriptions.push(sub).is_ok() {
                    info!("socket {}: added new subscription: {:?}", id, sub.ip);
                } else {
                    warn!(
                        "socket {}: subscription list full, ignoring new subscription {:?}",
                        id, sub.ip
                    );
                }
            }
            Either::Second(payload) => {
                if let Ok(message) = Message::new(&payload) {
                    'recip: for recep in subscriptions.iter() {
                        info!("socket {}: sending notification to {:?}", id, recep.ip);

                        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
                        if let Err(e) = socket.connect((recep.ip, NOTIFICATION_PORT)).await {
                            warn!(
                                "socket {}: failed to connect to {:?}: {:?}",
                                id, recep.ip, e
                            );
                            continue 'recip;
                        }

                        let message_bytes = match message.to_bytes() {
                            Ok(bytes) => bytes,
                            Err(_) => {
                                warn!("socket {}: failed to serialize message", id);
                                continue 'recip;
                            }
                        };

                        if let Err(e) = socket.write_all(&message_bytes).await {
                            warn!(
                                "socket {}: failed to send notification to {:?}: {:?}",
                                id, recep.ip, e
                            );
                        }
                    }
                } else {
                    warn!("socket {}: failed to create message", id);
                }
            }
        }
    }
}
