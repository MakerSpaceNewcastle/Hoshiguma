use crate::{
    DeviceCommunicator,
    devices::{
        compressor::CompressorInterfaceChannel, coolant_pump::CoolantPumpInterfaceChannel,
        coolant_rate_sensors::CoolantRateInterfaceChannel,
        radiator_fan::RadiatorFanInterfaceChannel,
        temperature_sensors::TemperatureInterfaceChannel,
    },
};
use defmt::warn;
use embassy_net::Stack;
use embassy_time::Instant;
use hoshiguma_api::{
    Message,
    cooler::{Request, Response, ResponseData},
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
            Request::GetGitRevision => Response(Ok(ResponseData::GitRevision(
                git_version::git_version!().try_into().unwrap(),
            ))),
            Request::GetUptime => Response(Ok(ResponseData::Uptime(
                Instant::now().duration_since(Instant::MIN).into(),
            ))),
            Request::GetBootReason => Response(Ok(ResponseData::BootReason(crate::boot_reason()))),
            Request::GetRadiatorFanState => Response(
                comm.radiator_fan
                    .get()
                    .await
                    .map(ResponseData::RadiatorFanState)
                    .map_err(|_| ()),
            ),
            Request::SetRadiatorFanState(state) => Response(
                comm.radiator_fan
                    .set(state)
                    .await
                    .map(ResponseData::RadiatorFanState)
                    .map_err(|_| ()),
            ),
            Request::GetCompressorState => Response(
                comm.compressor
                    .get()
                    .await
                    .map(ResponseData::CompressorState)
                    .map_err(|_| ()),
            ),
            Request::SetCompressorState(state) => Response(
                comm.compressor
                    .set(state)
                    .await
                    .map(ResponseData::CompressorState)
                    .map_err(|_| ()),
            ),
            Request::GetCoolantPumpState => Response(
                comm.coolant_pump
                    .get()
                    .await
                    .map(ResponseData::CoolantPumpState)
                    .map_err(|_| ()),
            ),
            Request::SetCoolantPumpState(state) => Response(
                comm.coolant_pump
                    .set(state)
                    .await
                    .map(ResponseData::CoolantPumpState)
                    .map_err(|_| ()),
            ),
            Request::GetTemperatures => Response(
                comm.temperatures
                    .get()
                    .await
                    .map(ResponseData::Temperatures)
                    .map_err(|_| ()),
            ),
            Request::GetCoolantFlowRate => Response(
                comm.coolant_flow_rate
                    .get()
                    .await
                    .map(ResponseData::CoolantFlowRate)
                    .map_err(|_| ()),
            ),
            Request::GetCoolantReturnRate => Response(
                comm.coolant_return_rate
                    .get()
                    .await
                    .map(ResponseData::CoolantReturnRate)
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
