use crate::mqtt::Mqtt;
use embassy_time::{Duration, Ticker};
use embedded_svc::wifi::{AuthMethod, ClientConfiguration, Configuration};
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{BlockingWifi, EspWifi},
};
use log::{error, info};

pub(crate) fn setup(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
) -> Box<BlockingWifi<EspWifi<'static>>> {
    let nvs = EspDefaultNvsPartition::take().expect("should have nvs partition");

    let mut wifi = BlockingWifi::wrap(
        EspWifi::new(modem, sysloop.clone(), Some(nvs)).expect("should have wifi"),
        sysloop,
    )
    .expect("should have wifi");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))
        .expect("empty wifi config should be set");

    info!("Starting WiFi");
    wifi.start().expect("wifi should be started");

    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: ssid.into(),
        password: pass.into(),
        auth_method: AuthMethod::WPA2Personal,
        ..Default::default()
    }))
    .expect("wifi config should be set");

    Box::new(wifi)
}

pub(crate) async fn task(
    mut wifi: Box<BlockingWifi<EspWifi<'static>>>,
    mqtt: Mqtt,
    led: crate::led::Led,
) {
    let mut ticker = Ticker::every(Duration::from_secs(15));

    loop {
        info!("WiFi");

        if !wifi.is_connected().unwrap() {
            led.set(crate::led::RED);

            info!("WiFi disconnected, connecting");
            match wifi.connect() {
                Ok(_) => info!("WiFi connected"),
                Err(e) => {
                    error!("WiFi connect failed: {:?}", e);
                    continue;
                }
            }

            info!("Waiting for network setup");
            match wifi.wait_netif_up() {
                Ok(_) => info!("Network is up"),
                Err(e) => {
                    error!("Network setup failed: {:?}", e);
                    continue;
                }
            }

            led.set(crate::led::GREEN);
            mqtt.publish_online();

            match wifi.wifi().sta_netif().get_ip_info() {
                Ok(info) => info!("DHCP info: {:?}", info),
                Err(e) => error!("Failed to get DHCP info: {:?}", e),
            }
        }

        ticker.next().await;
    }
}
