use crate::{
    network::{NetworkEvent, NETWORK_EVENTS},
    telemetry::TELEMETRY_MESSAGES,
};
use const_format::formatcp;
use core::fmt::Write;
use defmt::{debug, info, warn};
use embassy_futures::select::{select, Either};
use embassy_net::{tcp::TcpSocket, IpAddress, Ipv4Address, Stack};
use embassy_sync::pubsub::WaitResult;
use embassy_time::{Duration, Instant, Ticker};
use hoshiguma_protocol::peripheral_controller::{
    event::{ControlEvent, Event, EventKind, ObservationEvent},
    types::{
        AirAssistDemand, AirAssistPump, ChassisIntrusion, CoolantResevoirLevel, FumeExtractionFan,
        FumeExtractionMode, LaserEnable, MachineEnable, MachineOperationLockout, MachinePower,
        MachineRun, TemperatureReading,
    },
};
use rust_mqtt::{
    client::{
        client::MqttClient,
        client_config::{ClientConfig, MqttVersion},
    },
    packet::v5::{publish_packet::QualityOfService, reason_codes::ReasonCode},
    utils::rng_generator::CountingRng,
};

pub(crate) const BROKER_IP: IpAddress = IpAddress::Ipv4(Ipv4Address::new(192, 168, 8, 183));
const BROKER_PORT: u16 = 1883;

const CLIENT_ID: &str = "hoshiguma-telemetry-module";
const USERNAME: &str = "hoshiguma";

const TOPIC_ROOT: &str = "hoshiguma";
const TOPIC_ROOT_TELEMETRY_MODULE: &str = formatcp!("{TOPIC_ROOT}/telemetry-module");
const TOPIC_TELEMETRY_MODULE_ONLINE: &str = formatcp!("{TOPIC_ROOT_TELEMETRY_MODULE}/online");
const TOPIC_TELEMETRY_MODULE_VERSION: &str = formatcp!("{TOPIC_ROOT_TELEMETRY_MODULE}/version");

trait ClientExt {
    async fn publish(&mut self, topic: &str, payload: &[u8], retain: bool) -> Result<(), ()>;

    async fn publish_telem_value_str<'a>(
        &mut self,
        topic: &'a str,
        payload: &'a str,
    ) -> Result<(), ()>;

    async fn publish_telem_value_bool(&mut self, topic: &str, payload: bool) -> Result<(), ()>;

    async fn publish_telem_value_temperature(
        &mut self,
        topic: &str,
        payload: TemperatureReading,
    ) -> Result<(), ()>;

    async fn publish_telem_value_float(&mut self, topic: &str, payload: f32) -> Result<(), ()>;
}

impl<T: embedded_io_async::Read + embedded_io_async::Write, R: rand::RngCore> ClientExt
    for MqttClient<'_, T, 5, R>
{
    async fn publish(&mut self, topic: &str, payload: &[u8], retain: bool) -> Result<(), ()> {
        let start = Instant::now();

        let result = self
            .send_message(topic, payload, QualityOfService::QoS1, retain)
            .await;

        let end = Instant::now();
        let time_taken = end - start;
        info!("MQTT message send took {}ms", time_taken.as_millis());

        match result {
            Ok(_) => Ok(()),
            Err(ReasonCode::NoMatchingSubscribers) => Ok(()),
            Err(e) => {
                warn!("MQTT publish error: {:?}", e);
                Err(())
            }
        }
    }

    async fn publish_telem_value_str<'a>(
        &mut self,
        topic: &'a str,
        payload: &'a str,
    ) -> Result<(), ()> {
        self.publish(topic, payload.as_bytes(), true).await
    }

    async fn publish_telem_value_bool(&mut self, topic: &str, payload: bool) -> Result<(), ()> {
        self.publish(
            topic,
            match payload {
                true => b"true",
                false => b"false",
            },
            true,
        )
        .await
    }

    async fn publish_telem_value_temperature(
        &mut self,
        topic: &str,
        payload: TemperatureReading,
    ) -> Result<(), ()> {
        match payload {
            Ok(payload) => self.publish_telem_value_float(topic, payload).await,
            Err(_) => self.publish_telem_value_str(topic, "null").await,
        }
    }

    async fn publish_telem_value_float(&mut self, topic: &str, payload: f32) -> Result<(), ()> {
        let mut s = heapless::String::<16>::new();
        s.write_fmt(format_args!("{payload}")).unwrap();
        self.publish(topic, s.as_bytes(), true).await
    }
}

const BUFFER_SIZE: usize = 512;

pub(super) async fn run_client(stack: Stack<'_>) -> Result<(), ()> {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut mqtt_rx_buffer = [0; BUFFER_SIZE];
    let mut mqtt_tx_buffer = [0; BUFFER_SIZE];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!("Connecting to MQTT broker {}:{}", BROKER_IP, BROKER_PORT);
    socket
        .connect((BROKER_IP, BROKER_PORT))
        .await
        .map_err(|e| {
            warn!("Broker socket connection error: {:?}", e);
        })?;

    let mut client = {
        let mut config = ClientConfig::new(MqttVersion::MQTTv5, CountingRng(20000));
        config.add_client_id(CLIENT_ID);
        config.add_username(USERNAME);
        config.add_password(env!("MQTT_PASSWORD"));
        config.max_packet_size = BUFFER_SIZE as u32;
        config.add_will(TOPIC_TELEMETRY_MODULE_ONLINE, b"false", true);

        MqttClient::<_, 5, _>::new(
            socket,
            &mut mqtt_tx_buffer,
            BUFFER_SIZE,
            &mut mqtt_rx_buffer,
            BUFFER_SIZE,
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
        .publish_telem_value_str(TOPIC_TELEMETRY_MODULE_ONLINE, "true")
        .await?;

    client
        .publish_telem_value_str(TOPIC_TELEMETRY_MODULE_VERSION, git_version::git_version!())
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
            Either::Second(event) => match event {
                WaitResult::Lagged(msg_count) => {
                    warn!(
                        "Telemetry message receiver lagged, missed {} messages",
                        msg_count
                    );
                }
                WaitResult::Message(event) => {
                    publish_telemetry_event(&mut client, event).await?;
                }
            },
        }
    }
}

async fn publish_telemetry_event<
    W: embedded_io_async::Read + embedded_io_async::Write,
    R: rand::RngCore,
>(
    client: &mut MqttClient<'_, W, 5, R>,
    event: Event,
) -> Result<(), ()> {
    const ROOT: &str = formatcp!("{TOPIC_ROOT}/telemetry");

    {
        let mut time_since_boot = heapless::String::<16>::new();
        time_since_boot
            .write_fmt(format_args!("{}", event.timestamp_milliseconds))
            .unwrap();
        client
            .publish_telem_value_str(
                formatcp!("{ROOT}/controller/uptime_millis"),
                &time_since_boot,
            )
            .await?;
    }

    match event.kind {
        EventKind::MonitorsChanged(event) => {
            match serde_json_core::to_vec::<_, BUFFER_SIZE>(&event) {
                Ok(data) => {
                    client
                        .publish(formatcp!("{ROOT}/monitors"), &data, true)
                        .await
                }
                Err(e) => {
                    warn!("Cannot JSON serialise message: {}", e);
                    Err(())
                }
            }
        }
        EventKind::LockoutChanged(event) => {
            client
                .publish_telem_value_str(
                    formatcp!("{ROOT}/lockout"),
                    match event {
                        MachineOperationLockout::Permitted => "permitted",
                        MachineOperationLockout::PermittedUntilIdle => "permitted_until_idle",
                        MachineOperationLockout::Denied => "denied",
                    },
                )
                .await
        }
        EventKind::Observation(event) => match event {
            ObservationEvent::AirAssistDemand(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/air_assist_demand"),
                        match event {
                            AirAssistDemand::Idle => "idle",
                            AirAssistDemand::Demand => "demand",
                        },
                    )
                    .await
            }
            ObservationEvent::ChassisIntrusion(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/chassis_intrusion"),
                        match event {
                            ChassisIntrusion::Normal => "normal",
                            ChassisIntrusion::Intruded => "intruded",
                        },
                    )
                    .await
            }
            ObservationEvent::CoolantResevoirLevel(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/coolant_resevoir_level"),
                        match event {
                            Ok(CoolantResevoirLevel::Full) => "full",
                            Ok(CoolantResevoirLevel::Low) => "low",
                            Ok(CoolantResevoirLevel::Empty) => "empty",
                            Err(_) => "unknown",
                        },
                    )
                    .await
            }
            ObservationEvent::FumeExtractionMode(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/fume_extraction_mode"),
                        match event {
                            FumeExtractionMode::Automatic => "automatic",
                            FumeExtractionMode::OverrideRun => "override_run",
                        },
                    )
                    .await
            }
            ObservationEvent::MachinePower(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/machine_power"),
                        match event {
                            MachinePower::On => "on",
                            MachinePower::Off => "off",
                        },
                    )
                    .await
            }
            ObservationEvent::MachineRun(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/machine_run"),
                        match event {
                            MachineRun::Idle => "idle",
                            MachineRun::Running => "running",
                        },
                    )
                    .await
            }
            ObservationEvent::Temperatures(event) => {
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/onboard"),
                        event.onboard,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/ambient"),
                        event.ambient,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_flow"),
                        event.coolant_flow,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_return"),
                        event.coolant_return,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_pump"),
                        event.coolant_pump,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/laser_chamber"),
                        event.laser_chamber,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/electronics_bay_top"),
                        event.electronics_bay_top,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_resevoir_top"),
                        event.coolant_resevoir_top,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_resevoir_bottom"),
                        event.coolant_resevoir_bottom,
                    )
                    .await
            }
        },
        EventKind::Control(event) => match event {
            ControlEvent::AirAssistPump(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/air_assist_pump"),
                        match event {
                            AirAssistPump::Idle => "idle",
                            AirAssistPump::Run => "running",
                        },
                    )
                    .await
            }
            ControlEvent::FumeExtractionFan(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/fume_extraction_fan"),
                        match event {
                            FumeExtractionFan::Idle => "idle",
                            FumeExtractionFan::Run => "running",
                        },
                    )
                    .await
            }
            ControlEvent::LaserEnable(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/laser_enable"),
                        match event {
                            LaserEnable::Inhibit => "inhibited",
                            LaserEnable::Enable => "enabled",
                        },
                    )
                    .await
            }
            ControlEvent::MachineEnable(event) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/machine_enable"),
                        match event {
                            MachineEnable::Inhibit => "inhibited",
                            MachineEnable::Enable => "enabled",
                        },
                    )
                    .await
            }
            ControlEvent::StatusLamp(event) => {
                client
                    .publish_telem_value_bool(formatcp!("{ROOT}/status_lamp/red"), event.red)
                    .await?;
                client
                    .publish_telem_value_bool(formatcp!("{ROOT}/status_lamp/amber"), event.amber)
                    .await?;
                client
                    .publish_telem_value_bool(formatcp!("{ROOT}/status_lamp/green"), event.green)
                    .await
            }
        },
        event => match serde_json_core::to_vec::<_, BUFFER_SIZE>(&event) {
            Ok(data) => {
                client
                    .publish(formatcp!("{ROOT}/events"), &data, false)
                    .await
            }
            Err(e) => {
                warn!("Cannot JSON serialise message: {}", e);
                Err(())
            }
        },
    }
}
