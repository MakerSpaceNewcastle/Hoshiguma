use crate::{
    devices::temperature::{
        TEMPERATURE_SENSOR_READING, onewire_sensor_to_named_temperature_sensor,
    },
    telemetry::queue_telemetry_data_point,
};
use defmt::{info, warn};
use embassy_futures::select::{Either, select};
use embassy_net::Stack;
use embassy_time::{Duration, Ticker};
use hoshiguma_api::{
    API_PORT, AirflowSensorMeasurement, COOLER_IP_ADDRESS, REAR_SENSOR_BOARD_IP_ADDRESS,
    cooler::CoolantRate,
};
use hoshiguma_common::{network::send_request, telemetry::format_influx_line};

crate::variable_watch!(coolant_flow_rate, CoolantRate, 1);
crate::variable_watch!(coolant_return_rate, CoolantRate, 1);
crate::variable_watch!(extraction_airflow, AirflowSensorMeasurement, 1);

const COOLANT_FLOW_PULSES_PER_LITRE: f64 = 400.0;
const COOLANT_RETURN_PULSES_PER_LITRE: f64 = 230.0;

#[embassy_executor::task]
pub(crate) async fn task(stack: Stack<'static>) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("remote device observations").await;

    let temperature_pub = TEMPERATURE_SENSOR_READING.publisher().unwrap();

    let mut tick_1s = Ticker::every(Duration::from_secs(1));
    let mut tick_5s = Ticker::every(Duration::from_secs(5));

    loop {
        match select(tick_1s.next(), tick_5s.next()).await {
            Either::First(_) => {
                info!("Observation 1s tick");

                // Coolant flow rate
                if let Ok(response) = send_request(
                    stack,
                    COOLER_IP_ADDRESS,
                    API_PORT,
                    5,
                    &hoshiguma_api::cooler::request::GetCoolantFlowRate,
                )
                .await
                    && let Ok(state) = response.0
                {
                    let rate = state.into_rate(COOLANT_FLOW_PULSES_PER_LITRE);
                    COOLANT_FLOW_RATE.sender().send(rate);

                    queue_telemetry_data_point(format_influx_line(
                        format_args!(
                            "coolant_flow_rate value={},raw_pulses={}",
                            rate.into_inner(),
                            state.pulses()
                        ),
                        crate::wall_time::now(),
                    ));
                }

                // Coolant return rate
                if let Ok(response) = send_request(
                    stack,
                    COOLER_IP_ADDRESS,
                    API_PORT,
                    5,
                    &hoshiguma_api::cooler::request::GetCoolantReturnRate,
                )
                .await
                    && let Ok(state) = response.0
                {
                    let rate = state.into_rate(COOLANT_RETURN_PULSES_PER_LITRE);
                    COOLANT_RETURN_RATE.sender().send(rate);

                    queue_telemetry_data_point(format_influx_line(
                        format_args!(
                            "coolant_return_rate value={},raw_pulses={}",
                            rate.into_inner(),
                            state.pulses()
                        ),
                        crate::wall_time::now(),
                    ));
                }

                // Fume extraction suction
                if let Ok(response) = send_request(
                    stack,
                    REAR_SENSOR_BOARD_IP_ADDRESS,
                    API_PORT,
                    5,
                    &hoshiguma_api::rear_sensor_board::request::GetExtractionAirflow,
                )
                .await
                    && let Ok(state) = response.0
                {
                    EXTRACTION_AIRFLOW.sender().send(state);

                    if let Ok(state) = state {
                        queue_telemetry_data_point(format_influx_line(
                            format_args!(
                                "extraction_airflow_suction value={},temperature={}",
                                state.differential_pressure, state.temperature,
                            ),
                            crate::wall_time::now(),
                        ));
                    }
                }
            }
            Either::Second(_) => {
                info!("Observation 5s tick");

                // Cooler
                match send_request(
                    stack,
                    COOLER_IP_ADDRESS,
                    API_PORT,
                    5,
                    &hoshiguma_api::cooler::request::GetTemperatures,
                )
                .await
                {
                    Ok(response) => {
                        if let Ok(temperatures) = response.0 {
                            for reading in temperatures.into_iter() {
                                let reading = onewire_sensor_to_named_temperature_sensor(reading);
                                temperature_pub.publish(reading).await;
                            }
                        } else {
                            warn!("Reading failed");
                        }
                    }
                    Err(e) => {
                        warn!("Request failed: {}", e);
                    }
                }

                // Rear sensor board
                match send_request(
                    stack,
                    REAR_SENSOR_BOARD_IP_ADDRESS,
                    API_PORT,
                    5,
                    &hoshiguma_api::rear_sensor_board::request::GetTemperatures,
                )
                .await
                {
                    Ok(response) => {
                        if let Ok(temperatures) = response.0 {
                            for reading in temperatures.into_iter() {
                                let reading = onewire_sensor_to_named_temperature_sensor(reading);
                                temperature_pub.publish(reading).await;
                            }
                        } else {
                            warn!("Reading failed");
                        }
                    }
                    Err(e) => {
                        warn!("Request failed: {}", e);
                    }
                }
            }
        }
    }
}
