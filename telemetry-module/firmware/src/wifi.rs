use crate::telemetry::TELEMETRY_MESSAGES;
use cyw43::{PowerManagementMode, State};
use cyw43_pio::PioSpi;
use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_net::{
    dns::DnsQueryType, tcp::TcpSocket, Config, IpAddress, Stack, StackResources, StaticConfigV4,
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
use embassy_time::{Duration, Timer};
use rand::RngCore;
use rust_mqtt::{
    client::{
        client::MqttClient,
        client_config::{ClientConfig, MqttVersion},
    },
    packet::v5::publish_packet::QualityOfService,
    utils::rng_generator::CountingRng,
};
use static_cell::StaticCell;

// TODO
#[allow(dead_code)]
#[derive(Clone)]
pub(crate) enum NetworkEvent {
    NetworkConnected(StaticConfigV4),

    MqttBrokerConnected(IpAddress),
    MqttBrokerDisconnected,
}

pub(crate) static NETWORK_EVENTS: Channel<CriticalSectionRawMutex, NetworkEvent, 16> =
    Channel::new();

bind_interrupts!(struct Irqs {
    PIO0_IRQ_0 => InterruptHandler<PIO0>;
});

#[embassy_executor::task]
pub(super) async fn task(r: crate::WifiResources, spawner: Spawner) {
    let pwr = Output::new(r.pwr, Level::Low);
    let cs = Output::new(r.cs, Level::High);

    let mut pio = Pio::new(r.pio, Irqs);

    let spi = PioSpi::new(
        &mut pio.common,
        pio.sm0,
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

    static STACK: StaticCell<Stack<cyw43::NetDriver<'static>>> = StaticCell::new();
    static RESOURCES: StaticCell<StackResources<4>> = StaticCell::new();
    let stack = &*STACK.init(Stack::new(
        net_device,
        Config::dhcpv4(Default::default()),
        RESOURCES.init(StackResources::<4>::new()),
        seed,
    ));

    unwrap!(spawner.spawn(net_task(stack)));

    let mut rx_buffer = [0; 4096];
    let mut tx_buffer = [0; 4096];

    const MQTT_BUFFER_SIZE: usize = 512;
    let mut mqtt_rx_buffer = [0; MQTT_BUFFER_SIZE];
    let mut mqtt_tx_buffer = [0; MQTT_BUFFER_SIZE];

    let mut telem_rx = TELEMETRY_MESSAGES.subscriber().unwrap();

    info!("Joining WiFi network");
    loop {
        match control.join_wpa2("Maker Space", "TODO").await {
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
        let mut socket = TcpSocket::new(stack, &mut rx_buffer, &mut tx_buffer);
        socket.set_timeout(Some(Duration::from_secs(10)));

        let broker_address = match stack
            .dns_query("broker.hivemq.com", DnsQueryType::A)
            .await
            .map(|a| a[0])
        {
            Ok(address) => address,
            Err(e) => {
                info!("DNS lookup error: {}", e);
                continue;
            }
        };

        let broker_endpoint = (broker_address, 1883);

        info!("Connecting to MQTT broker");
        let connection = socket.connect(broker_endpoint).await;
        if let Err(e) = connection {
            warn!("Broker socket connection error: {:?}", e);
            continue;
        }

        let mut client = {
            let mut config = ClientConfig::new(MqttVersion::MQTTv5, CountingRng(20000));
            config.add_client_id("doot");
            config.max_packet_size = MQTT_BUFFER_SIZE as u32;
            config.add_will("TODO/telemetry-module/online", b"false", true);

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
                NETWORK_EVENTS
                    .send(NetworkEvent::MqttBrokerConnected(broker_address))
                    .await;
            }
            Err(e) => {
                warn!("MQTT error: {:?}", e);
                NETWORK_EVENTS
                    .send(NetworkEvent::MqttBrokerDisconnected)
                    .await;
                continue;
            }
        }

        match client
            .send_message(
                "TODO/telemetry-module/online",
                b"true",
                QualityOfService::QoS1,
                true,
            )
            .await
        {
            Ok(()) => {}
            Err(e) => {
                warn!("MQTT error: {:?}", e);
                NETWORK_EVENTS
                    .send(NetworkEvent::MqttBrokerDisconnected)
                    .await;
                continue;
            }
        }

        match client
            .send_message(
                "TODO/telemetry-module/version",
                crate::git_version_string().as_bytes(),
                QualityOfService::QoS1,
                true,
            )
            .await
        {
            Ok(()) => {}
            Err(e) => {
                warn!("MQTT error: {:?}", e);
                NETWORK_EVENTS
                    .send(NetworkEvent::MqttBrokerDisconnected)
                    .await;
                continue;
            }
        }

        loop {
            match telem_rx.next_message().await {
                WaitResult::Lagged(msg_count) => {
                    warn!(
                        "Telemetry message receiver lagged, missed {} messages",
                        msg_count
                    );
                }
                WaitResult::Message(msg) => {
                    let veccy = serde_json_core::to_vec::<_, MQTT_BUFFER_SIZE>(&msg);
                    match veccy {
                        Ok(data) => {
                            match client
                                .send_message(
                                    "TODO/controller-telemetry",
                                    &data,
                                    QualityOfService::QoS1,
                                    true,
                                )
                                .await
                            {
                                Ok(()) => {}
                                Err(e) => {
                                    warn!("MQTT error: {:?}", e);
                                    NETWORK_EVENTS
                                        .send(NetworkEvent::MqttBrokerDisconnected)
                                        .await;
                                    continue;
                                }
                            }
                        }
                        Err(e) => warn!("Cannot JSON serialise message: {}", e),
                    }
                }
            }
        }
    }
}

#[embassy_executor::task]
async fn cyw43_task(
    runner: cyw43::Runner<'static, Output<'static>, PioSpi<'static, PIO0, 0, DMA_CH0>>,
) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn net_task(stack: &'static Stack<cyw43::NetDriver<'static>>) -> ! {
    stack.run().await
}
