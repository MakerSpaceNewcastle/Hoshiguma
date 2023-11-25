use embedded_svc::mqtt::client::QoS;
use esp_idf_svc::mqtt::client::{EspMqttClient, LwtConfiguration, MqttClientConfiguration};
use log::{error, info};
use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

#[derive(Clone)]
pub(crate) struct MqttService {
    client: Arc<Mutex<EspMqttClient>>,
}

impl MqttService {
    pub(crate) fn new() -> Self {
        let client = EspMqttClient::new(
            "mqtt://broker.hivemq.com",
            &MqttClientConfiguration {
                keep_alive_interval: Some(Duration::from_secs(3)),
                reconnect_timeout: Some(Duration::from_secs(1)),
                lwt: Some(LwtConfiguration {
                    topic: satori_mqtt_config::TOPIC_ALIVE,
                    qos: QoS::ExactlyOnce,
                    retain: false,
                    payload: satori_mqtt_config::ALIVE_PAYLOAD_OFFLINE,
                }),
                ..Default::default()
            },
            |event| {
                // TODO
                info!("mqtt: {:?}", event);
            },
        )
        .expect("client should be created");

        Self {
            client: Arc::new(Mutex::new(client)),
        }
    }

    pub(crate) fn publish_online(&self) {
        if let Err(e) = self.client.lock().unwrap().publish(
            satori_mqtt_config::TOPIC_ALIVE,
            QoS::ExactlyOnce,
            false,
            satori_mqtt_config::ALIVE_PAYLOAD_ONLINE,
        ) {
            error!("publish online failed: {:?}", e);
        }
    }

    pub(crate) fn test(&self) {
        if let Err(e) = self.client.lock().unwrap().publish(
            satori_mqtt_config::TOPIC_STATUS,
            QoS::ExactlyOnce,
            false,
            "doot".as_bytes(),
        ) {
            error!("publish todo failed: {:?}", e);
        }
    }
}
