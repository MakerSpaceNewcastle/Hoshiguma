use crate::{logic::interlock::update_monitor_severity, telemetry::queue_telemetry_data_point};
use defmt::info;
use embassy_net::Stack;
use embassy_time::{Instant, Timer};
use hoshiguma_api::{COOLER_IP_ADDRESS, HMI_IP_ADDRESS, Monitor, REAR_SENSOR_BOARD_IP_ADDRESS};
use hoshiguma_common::remote_device_healthcheck::RemoteDeviceHealthCheck;

#[embassy_executor::task]
pub(crate) async fn task(stack: Stack<'static>) -> ! {
    #[cfg(feature = "trace")]
    crate::trace::name_task("remote device monitor").await;

    let mut cooler = RemoteDeviceHealthCheck::<
        hoshiguma_api::cooler::request::GetSystemInformation,
        _,
        _,
        _,
    >::new(
        stack,
        "cooler",
        COOLER_IP_ADDRESS,
        async |severity| {
            update_monitor_severity(Monitor::CoolerCommunication, severity).await;
        },
        |data_point| {
            queue_telemetry_data_point(data_point);
        },
    );

    let mut rear_sensor_board = RemoteDeviceHealthCheck::<
        hoshiguma_api::rear_sensor_board::request::GetSystemInformation,
        _,
        _,
        _,
    >::new(
        stack,
        "rear_sensor_board",
        REAR_SENSOR_BOARD_IP_ADDRESS,
        async |severity| {
            update_monitor_severity(Monitor::RearSensorBoardCommunication, severity).await;
        },
        |data_point| {
            queue_telemetry_data_point(data_point);
        },
    );

    let mut hmi = RemoteDeviceHealthCheck::<
        hoshiguma_api::hmi::to_hmi::request::GetSystemInformation,
        _,
        _,
        _,
    >::new(
        stack,
        "hmi",
        HMI_IP_ADDRESS,
        async |severity| {
            update_monitor_severity(Monitor::HmiCommunication, severity).await;
        },
        |data_point| {
            queue_telemetry_data_point(data_point);
        },
    );

    loop {
        info!("Checking remote devices... {}", Instant::now().as_millis());
        cooler.check().await;
        rear_sensor_board.check().await;
        hmi.check().await;
        info!(
            "Checking remote devices done {}",
            Instant::now().as_millis()
        );

        Timer::after_secs(2).await;
    }
}
