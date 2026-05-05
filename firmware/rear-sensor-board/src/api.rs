use crate::{
    DeviceCommunicator,
    devices::{
        airflow_sensor::AirflowSensorInterfaceChannel, status_light::StatusLightInterfaceChannel,
        temperature_sensors::TemperatureInterfaceChannel,
    },
};
use defmt::warn;
use embassy_net::Stack;
use embassy_time::Instant;
use hoshiguma_api::{
    Message, SystemInformation,
    rear_sensor_board::{Request, Response, ResponseData},
};
use hoshiguma_common::network::message_handler_loop;

pub(crate) const NUM_LISTENERS: usize = 3;

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
pub(super) async fn task(stack: Stack<'static>, id: usize, mut comm: DeviceCommunicator) {
    message_handler_loop(stack, id, async |mut message| {
        let request = match message.payload::<Request>() {
            Ok(request) => request,
            Err(_) => {
                warn!("socket {}: failed to parse request", id);
                return Message::new(&Response(Err(()))).unwrap();
            }
        };

        let _ = crate::COMM_GOOD_INDICATOR.try_send(());

        let response = match request {
            Request::GetSystemInformation => {
                Response(Ok(ResponseData::SystemInformation(SystemInformation {
                    git_revision: git_version::git_version!().try_into().unwrap(),
                    uptime: Instant::now().duration_since(Instant::MIN).into(),
                    boot_reason: crate::boot_reason(),
                })))
            }
            Request::SetStatusLight(settings) => Response(
                comm.status_light
                    .set(settings)
                    .await
                    .map(ResponseData::StatusLightSettings)
                    .map_err(|_| ()),
            ),
            Request::GetExtractionAirflow => Response(
                comm.airflow
                    .get()
                    .await
                    .map(ResponseData::ExtractionAriflow)
                    .map_err(|_| ()),
            ),
            Request::GetTemperatures => Response(
                comm.temperatures
                    .get()
                    .await
                    .map(ResponseData::Temperatures)
                    .map_err(|_| ()),
            ),
        };

        match Message::new(&response) {
            Ok(message) => message,
            Err(_) => {
                warn!("socket {}: failed to serialize response", id);
                Message::new(&Response(Err(()))).unwrap()
            }
        }
    })
    .await
}
