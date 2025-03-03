pub(crate) mod mqtt;

use cyw43::{JoinOptions, PowerManagementMode, State};
use cyw43_pio::{PioSpi, DEFAULT_CLOCK_DIVIDER};
use defmt::{info, unwrap, warn};
use embassy_executor::Spawner;
use embassy_net::{Config, StackResources, StaticConfigV4};
use embassy_rp::{
    bind_interrupts,
    clocks::RoscRng,
    gpio::{Level, Output},
    peripherals::{DMA_CH0, PIO0},
    pio::{InterruptHandler, Pio},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Timer;
use rand::RngCore;
use static_cell::StaticCell;

const WIFI_SSID: &str = "Maker Space";

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

    let fw = include_bytes!("../../cyw43-firmware/43439A0.bin");
    let (net_device, mut control, runner) = cyw43::new(state, pwr, spi, fw).await;
    unwrap!(spawner.spawn(cyw43_task(runner)));

    let clm = include_bytes!("../../cyw43-firmware/43439A0_clm.bin");
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
        if mqtt::run_client(stack).await.is_err() {
            // Notify of MQTT broker connection loss
            NETWORK_EVENTS
                .send(NetworkEvent::MqttBrokerDisconnected)
                .await;
        }

        // Wait a little bit of time before connecting again
        Timer::after_millis(500).await;
    }
}
