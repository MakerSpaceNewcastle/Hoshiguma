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
    Message, SystemInformation,
    cooler::{request, response},
};
use hoshiguma_common::network::message_handler_loop;

pub(crate) const NUM_LISTENERS: usize = 3;

#[embassy_executor::task(pool_size = NUM_LISTENERS)]
pub(super) async fn task(stack: Stack<'static>, id: usize, mut comm: DeviceCommunicator) {
    message_handler_loop(stack, id, async |mut message| {
        let response = if message.payload::<request::GetSystemInformation>().is_ok() {
            Message::new(&response::SystemInformation(SystemInformation {
                git_revision: git_version::git_version!().try_into().unwrap(),
                uptime: Instant::now().duration_since(Instant::MIN).into(),
                boot_reason: crate::boot_reason(),
            }))
            .ok()
        } else if let Ok(state) = message.payload::<request::SetRadiatorFanState>() {
            Message::new(&response::RadiatorFanState(
                comm.radiator_fan.set(state.0).await.map_err(|_| ()),
            ))
            .ok()
        } else if let Ok(state) = message.payload::<request::SetCompressorState>() {
            Message::new(&response::CompressorState(
                comm.compressor.set(state.0).await.map_err(|_| ()),
            ))
            .ok()
        } else if let Ok(state) = message.payload::<request::SetCoolantPumpState>() {
            Message::new(&response::CoolantPumpState(
                comm.coolant_pump.set(state.0).await.map_err(|_| ()),
            ))
            .ok()
        } else if message.payload::<request::GetTemperatures>().is_ok() {
            Message::new(&response::Temperatures(
                comm.temperatures.get().await.map_err(|_| ()),
            ))
            .ok()
        } else if message.payload::<request::GetCoolantFlowRate>().is_ok() {
            Message::new(&response::CoolantFlowRate(
                comm.coolant_flow_rate.get_rate().await.map_err(|_| ()),
            ))
            .ok()
        } else if message.payload::<request::GetCoolantFlowPulses>().is_ok() {
            Message::new(&response::CoolantFlowPulses(
                comm.coolant_flow_rate
                    .get_total_pulses()
                    .await
                    .map_err(|_| ()),
            ))
            .ok()
        } else if message.payload::<request::GetCoolantReturnRate>().is_ok() {
            Message::new(&response::CoolantReturnRate(
                comm.coolant_return_rate.get_rate().await.map_err(|_| ()),
            ))
            .ok()
        } else if message.payload::<request::GetCoolantReturnPulses>().is_ok() {
            Message::new(&response::CoolantReturnPulses(
                comm.coolant_return_rate
                    .get_total_pulses()
                    .await
                    .map_err(|_| ()),
            ))
            .ok()
        } else {
            None
        };

        match response {
            Some(response) => {
                // Indicate that good communication has happened
                let _ = crate::COMM_GOOD_INDICATOR.try_send(());

                response
            }
            None => {
                warn!("API error, no response created");
                // Return an API error if no response was generated
                Message::new(&response::ApiError).expect("API error failed to serialise")
            }
        }
    })
    .await
}
