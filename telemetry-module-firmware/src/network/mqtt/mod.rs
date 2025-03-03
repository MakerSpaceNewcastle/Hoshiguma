pub(crate) mod config;

use crate::{
    network::{NetworkEvent, NETWORK_EVENTS},
    telemetry::TELEMETRY_MESSAGES,
};
use defmt::{debug, info, warn};
use embassy_futures::select::{select, Either};
use embassy_net::{tcp::TcpSocket, Stack};
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Ticker};
use rand::RngCore;
use rust_mqtt::{
    client::{
        client::MqttClient,
        client_config::{ClientConfig, MqttVersion},
    },
    packet::v5::{publish_packet::QualityOfService, reason_codes::ReasonCode},
    utils::rng_generator::CountingRng,
};

trait ClientExt {
    async fn publish<'a>(
        &mut self,
        topic: &'a str,
        payload: &'a [u8],
        retain: bool,
    ) -> Result<(), ()>;
}

impl<T: embedded_io_async::Read + embedded_io_async::Write, R: RngCore> ClientExt
    for MqttClient<'_, T, 5, R>
{
    async fn publish<'a>(
        &mut self,
        topic: &'a str,
        payload: &'a [u8],
        retain: bool,
    ) -> Result<(), ()> {
        let result = self
            .send_message(topic, payload, QualityOfService::QoS1, retain)
            .await;

        match result {
            Ok(_) => Ok(()),
            Err(ReasonCode::NoMatchingSubscribers) => Ok(()),
            Err(e) => {
                warn!("MQTT publish error: {:?}", e);
                Err(())
            }
        }
    }
}

const MQTT_BUFFER_SIZE: usize = 512;

pub(super) async fn run_client(stack: Stack<'_>) -> Result<(), ()> {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut mqtt_rx_buffer = [0; MQTT_BUFFER_SIZE];
    let mut mqtt_tx_buffer = [0; MQTT_BUFFER_SIZE];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!(
        "Connecting to MQTT broker {}:{}",
        config::MQTT_BROKER_IP,
        config::MQTT_BROKER_PORT
    );
    socket
        .connect((config::MQTT_BROKER_IP, config::MQTT_BROKER_PORT))
        .await
        .map_err(|e| {
            warn!("Broker socket connection error: {:?}", e);
        })?;

    let mut client = {
        let mut config = ClientConfig::new(MqttVersion::MQTTv5, CountingRng(20000));
        config.add_client_id(config::MQTT_CLIENT_ID);
        config.add_username(config::MQTT_USERNAME);
        config.add_password(env!("MQTT_PASSWORD"));
        config.max_packet_size = MQTT_BUFFER_SIZE as u32;
        config.add_will(config::topics::TELEMETRY_MODULE_ONLINE, b"false", true);

        MqttClient::<_, 5, _>::new(
            socket,
            &mut mqtt_tx_buffer,
            MQTT_BUFFER_SIZE,
            &mut mqtt_rx_buffer,
            MQTT_BUFFER_SIZE,
            config,
        )
    };

    match client.connect_to_broker().await {
        Ok(()) => {
            info!("Connected to MQTT broker");
            NETWORK_EVENTS.send(NetworkEvent::MqttBrokerConnected).await;
        }
        Err(e) => {
            warn!("MQTT error: {:?}", e);
            return Err(());
        }
    }

    client
        .publish(config::topics::TELEMETRY_MODULE_ONLINE, b"true", true)
        .await?;

    client
        .publish(
            config::topics::TELEMETRY_MODULE_VERSION,
            git_version::git_version!().as_bytes(),
            true,
        )
        .await?;

    let mut ping_tick = Ticker::every(Duration::from_secs(5));
    let mut telem_rx = TELEMETRY_MESSAGES.subscriber().unwrap();

    loop {
        match select(ping_tick.next(), telem_rx.next_message()).await {
            Either::First(_) => match client.send_ping().await {
                Ok(_) => {
                    debug!("MQTT ping OK");
                }
                Err(e) => {
                    warn!("MQTT ping error: {:?}", e);
                    return Err(());
                }
            },
            Either::Second(msg) => match msg {
                WaitResult::Lagged(msg_count) => {
                    warn!(
                        "Telemetry message receiver lagged, missed {} messages",
                        msg_count
                    );
                }
                WaitResult::Message(msg) => {
                    match serde_json_core::to_vec::<_, MQTT_BUFFER_SIZE>(&msg) {
                        Ok(data) => {
                            client
                                .publish(config::topics::TELEMETRY_EVENTS, &data, false)
                                .await?;
                        }
                        Err(e) => warn!("Cannot JSON serialise message: {}", e),
                    }
                }
            },
        }
    }
}
