use crate::telemetry::TELEMETRY_MESSAGES;
use cyw43::{JoinOptions, PowerManagementMode, State};
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::{debug, info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_futures::select::{select, Either};
use embassy_net::{
    tcp::TcpSocket, Config, IpAddress, Ipv4Address, Stack, StackResources, StaticConfigV4,
};
use embassy_rp::{
    bind_interrupts,
    clocks::RoscRng,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIO0},
    pio::{InterruptHandler, Pio},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, pubsub::WaitResult,
};
use embassy_time::{Duration, Ticker, Timer};
use rand::RngCore;
use rust_mqtt::{
    client::{
        client::MqttClient,
        client_config::{ClientConfig, MqttVersion},
    },
    packet::v5::{publish_packet::QualityOfService, reason_codes::ReasonCode},
    utils::rng_generator::CountingRng,
};
use static_cell::StaticCell;

const WIFI_SSID: &str = "Maker Space";

pub(crate) const MQTT_BROKER_IP: IpAddress = IpAddress::Ipv4(Ipv4Address::new(192, 168, 8, 183));
const MQTT_BROKER_PORT: u16 = 1883;

const MQTT_CLIENT_ID: &str = "hoshiguma-telemetry-module";
const MQTT_USERNAME: &str = "hoshiguma";

const ONLINE_MQTT_TOPIC: &str = "hoshiguma/telemetry-module/online";
const VERSION_MQTT_TOPIC: &str = "hoshiguma/telemetry-module/version";
const TELEMETRY_MQTT_TOPIC: &str = "hoshiguma/events";

const MQTT_BUFFER_SIZE: usize = 512;

#[derive(Clone)]
pub(crate) enum NetworkEvent {
    NetworkConnected(StaticConfigV4),

    MqttBrokerConnected,
    MqttBrokerDisconnected,
}

pub(crate) static NETWORK_EVENTS: Channel<CriticalSectionRawMutex, NetworkEvent, 16> =
    Channel::new();

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, cyw43::NetDriver<'static>>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
pub(super) async fn task(r: crate::WifiResources, spawner: Spawner) {
    let pwr = Output::new(r.pwr, Level::Low);
    let cs = Output::new(r.cs, Level::High);

    let mut pio = Pio::new(r.pio, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
        DEFAULT_CLOCK_DIVIDER,
        pio.irq0,
        cs,
        r.dio,
        r.clk,
        r.dma_ch,
    );

    static STATE: StaticCell<State> = StaticCell::new();
    let state = STATE.init(State::new());

    let fw = include_bytes!("../cyw43-firmware/43439A0.bin");
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    let clm = include_bytes!("../cyw43-firmware/43439A0_clm.bin");
    control.init(clm).await;

    control
        .set_power_management(PowerManagementMode::PowerSave)
        .await;

    let mut rng = RoscRng;
    let seed = rng.next_u64();

    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(
        net_device,
        Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::<4>::new()),
        seed,
    );
    unwrap!(spawner.spawn(net_task(runner)));

    info!("Joining WiFi network {}", WIFI_SSID);
    loop {
        match control
            .join(
                WIFI_SSID,
                JoinOptions::new(env!("WIFI_PASSWORD").as_bytes()),
            )
            .await
        {
            Ok(_) => break,
            Err(err) => {
                warn!("Failed to join WiFi network with status {}", err.status);
            }
        }
    }

    // Get configuration via DHCP
    {
        info!("Waiting for DHCP");
        while !stack.is_config_up() {
            Timer::after_millis(100).await;
        }
        info!("DHCP is now up");

        let config = stack.config_v4().unwrap();
        NETWORK_EVENTS
            .send(NetworkEvent::NetworkConnected(config))
            .await;
    }

    loop {
        // Start the MQTT client
        if run_mqtt_client(stack).await.is_err() {
            // Notify of MQTT broker connection loss
            NETWORK_EVENTS
                .send(NetworkEvent::MqttBrokerDisconnected)
                .await;
        }

        // Wait a little bit of time before connecting again
        Timer::after_millis(500).await;
    }
}

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

async fn run_mqtt_client(stack: Stack<'_>) -> Result<(), ()> {
    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    let mut mqtt_rx_buffer = [0; MQTT_BUFFER_SIZE];
    let mut mqtt_tx_buffer = [0; MQTT_BUFFER_SIZE];

    let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
    socket.set_timeout(Some(Duration::from_secs(10)));

    info!(
        "Connecting to MQTT broker {}:{}",
        MQTT_BROKER_IP, MQTT_BROKER_PORT
    );
    socket
        .connect((MQTT_BROKER_IP, MQTT_BROKER_PORT))
        .await
        .map_err(|e| {
            warn!("Broker socket connection error: {:?}", e);
        })?;

    let mut client = {
        let mut config = ClientConfig::new(MqttVersion::MQTTv5, CountingRng(20000));
        config.add_client_id(MQTT_CLIENT_ID);
        config.add_username(MQTT_USERNAME);
        config.add_password(env!("MQTT_PASSWORD"));
        config.max_packet_size = MQTT_BUFFER_SIZE as u32;
        config.add_will(ONLINE_MQTT_TOPIC, b"false", true);

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

    client.publish(ONLINE_MQTT_TOPIC, b"true", true).await?;

    client
        .publish(
            VERSION_MQTT_TOPIC,
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
                            client.publish(TELEMETRY_MQTT_TOPIC, &data, false).await?;
                        }
                        Err(e) => warn!("Cannot JSON serialise message: {}", e),
                    }
                }
            },
        }
    }
}
