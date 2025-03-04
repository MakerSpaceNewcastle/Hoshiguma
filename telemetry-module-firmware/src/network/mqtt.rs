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
use hoshiguma_protocol::{
    payload::{
        control::{AirAssistPump, ControlPayload, FumeExtractionFan, LaserEnable, MachineEnable},
        observation::{
            AirAssistDemand, ChassisIntrusion, CoolantResevoirLevel, FumeExtractionMode,
            MachinePower, MachineRun, ObservationPayload, TemperatureReading,
        },
        process::{MachineOperationLockout, ProcessPayload},
        system::SystemMessagePayload,
        Payload,
    },
    Message,
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
            Either::Second(msg) => match msg {
                WaitResult::Lagged(msg_count) => {
                    warn!(
                        "Telemetry message receiver lagged, missed {} messages",
                        msg_count
                    );
                }
                WaitResult::Message(msg) => {
                    publish_telemetry_message(&mut client, msg).await?;
                }
            },
        }
    }
}

async fn publish_telemetry_message<
    W: embedded_io_async::Read + embedded_io_async::Write,
    R: rand::RngCore,
>(
    client: &mut MqttClient<'_, W, 5, R>,
    msg: Message,
) -> Result<(), ()> {
    const ROOT: &str = formatcp!("{TOPIC_ROOT}/telemetry");

    {
        let mut time_since_boot = heapless::String::<16>::new();
        time_since_boot
            .write_fmt(format_args!("{}", msg.millis_since_boot))
            .unwrap();
        client
            .publish_telem_value_str(
                formatcp!("{ROOT}/controller/uptime_millis"),
                &time_since_boot,
            )
            .await?;
    }

    match msg.payload {
        Payload::Observation(msg) => match msg {
            ObservationPayload::AirAssistDemand(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/air_assist_demand"),
                        match msg {
                            AirAssistDemand::Idle => "idle",
                            AirAssistDemand::Demand => "demand",
                        },
                    )
                    .await
            }
            ObservationPayload::ChassisIntrusion(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/chassis_intrusion"),
                        match msg {
                            ChassisIntrusion::Normal => "normal",
                            ChassisIntrusion::Intruded => "intruded",
                        },
                    )
                    .await
            }
            ObservationPayload::CoolantResevoirLevel(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/coolant_resevoir_level"),
                        match msg {
                            Ok(CoolantResevoirLevel::Full) => "full",
                            Ok(CoolantResevoirLevel::Low) => "low",
                            Ok(CoolantResevoirLevel::Empty) => "empty",
                            Err(_) => "unknown",
                        },
                    )
                    .await
            }
            ObservationPayload::FumeExtractionMode(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/fume_extraction_mode"),
                        match msg {
                            FumeExtractionMode::Automatic => "automatic",
                            FumeExtractionMode::OverrideRun => "override_run",
                        },
                    )
                    .await
            }
            ObservationPayload::MachinePower(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/machine_power"),
                        match msg {
                            MachinePower::On => "on",
                            MachinePower::Off => "off",
                        },
                    )
                    .await
            }
            ObservationPayload::MachineRun(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/machine_run"),
                        match msg {
                            MachineRun::Idle => "idle",
                            MachineRun::Running => "running",
                        },
                    )
                    .await
            }
            ObservationPayload::Temperatures(msg) => {
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/onboard"),
                        msg.onboard,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/ambient"),
                        msg.ambient,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_flow"),
                        msg.coolant_flow,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_return"),
                        msg.coolant_return,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_pump"),
                        msg.coolant_pump,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/laser_chamber"),
                        msg.laser_chamber,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/electronics_bay_top"),
                        msg.electronics_bay_top,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_resevoir_top"),
                        msg.coolant_resevoir_top,
                    )
                    .await?;
                client
                    .publish_telem_value_temperature(
                        formatcp!("{ROOT}/temperature/coolant_resevoir_bottom"),
                        msg.coolant_resevoir_bottom,
                    )
                    .await
            }
        },
        Payload::Process(msg) => match msg {
            ProcessPayload::Monitor(msg) => match serde_json_core::to_vec::<_, BUFFER_SIZE>(&msg) {
                Ok(data) => {
                    client
                        .publish(formatcp!("{ROOT}/monitor_change"), &data, true)
                        .await
                }
                Err(e) => {
                    warn!("Cannot JSON serialise message: {}", e);
                    Err(())
                }
            },
            ProcessPayload::Alarms(msg) => {
                match serde_json_core::to_vec::<_, BUFFER_SIZE>(&msg.alarms) {
                    Ok(data) => {
                        client
                            .publish(formatcp!("{ROOT}/alarms"), &data, true)
                            .await
                    }
                    Err(e) => {
                        warn!("Cannot JSON serialise message: {}", e);
                        Err(())
                    }
                }
            }
            ProcessPayload::Lockout(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/lockout"),
                        match msg {
                            MachineOperationLockout::Permitted => "permitted",
                            MachineOperationLockout::PermittedUntilIdle => "permitted_until_idle",
                            MachineOperationLockout::Denied => "denied",
                        },
                    )
                    .await
            }
        },
        Payload::Control(msg) => match msg {
            ControlPayload::AirAssistPump(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/air_assist_pump"),
                        match msg {
                            AirAssistPump::Idle => "idle",
                            AirAssistPump::Run => "running",
                        },
                    )
                    .await
            }
            ControlPayload::FumeExtractionFan(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/fume_extraction_fan"),
                        match msg {
                            FumeExtractionFan::Idle => "idle",
                            FumeExtractionFan::Run => "running",
                        },
                    )
                    .await
            }
            ControlPayload::LaserEnable(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/laser_enable"),
                        match msg {
                            LaserEnable::Inhibit => "inhibited",
                            LaserEnable::Enable => "enabled",
                        },
                    )
                    .await
            }
            ControlPayload::MachineEnable(msg) => {
                client
                    .publish_telem_value_str(
                        formatcp!("{ROOT}/machine_enable"),
                        match msg {
                            MachineEnable::Inhibit => "inhibited",
                            MachineEnable::Enable => "enabled",
                        },
                    )
                    .await
            }
            ControlPayload::StatusLamp(msg) => {
                client
                    .publish_telem_value_bool(formatcp!("{ROOT}/status_lamp/red"), msg.red)
                    .await?;
                client
                    .publish_telem_value_bool(formatcp!("{ROOT}/status_lamp/amber"), msg.amber)
                    .await?;
                client
                    .publish_telem_value_bool(formatcp!("{ROOT}/status_lamp/green"), msg.green)
                    .await
            }
        },
        Payload::System(SystemMessagePayload::Heartbeat(_)) => {
            // Ignore heartbeat messages
            Ok(())
        }
        msg => match serde_json_core::to_vec::<_, BUFFER_SIZE>(&msg) {
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
