use super::TemperaturesExt;
use defmt::Format;
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel, watch::Watch,
};
use hoshiguma_protocol::accessories::cooler::{
    rpc::Request as CoolerRequest,
    types::{
        CompressorState, CoolantFlow, CoolantPumpState, CoolantReservoirLevel, RadiatorFanState,
        Temperatures,
    },
};

#[derive(Debug, Clone, Format)]
pub(crate) enum CoolerControlCommand {
    RadiatorFan(RadiatorFanState),
    Compressor(CompressorState),
    CoolantPump(CoolantPumpState),
}

impl From<CoolerControlCommand> for CoolerRequest {
    fn from(cmd: CoolerControlCommand) -> Self {
        match cmd {
            CoolerControlCommand::RadiatorFan(radiator_fan) => Self::SetRadiatorFan(radiator_fan),
            CoolerControlCommand::Compressor(compressor) => Self::SetCompressor(compressor),
            CoolerControlCommand::CoolantPump(coolant_pump) => Self::SetCoolantPump(coolant_pump),
        }
    }
}

pub(crate) static COOLER_CONTROL_COMMAND: PubSubChannel<
    CriticalSectionRawMutex,
    CoolerControlCommand,
    8,
    1,
    2,
> = PubSubChannel::new();

pub(crate) static COOLANT_FLOW_READ: Watch<CriticalSectionRawMutex, CoolantFlow, 1> = Watch::new();

pub(crate) static COOLER_TEMPERATURES_READ: Watch<CriticalSectionRawMutex, Temperatures, 2> =
    Watch::new();

pub(crate) static COOLANT_RESEVOIR_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    CoolantReservoirLevel,
    1,
> = Watch::new();

impl TemperaturesExt for Temperatures {
    fn any_failed_sensors(&self) -> bool {
        let sensors = [
            &self.onboard,
            &self.internal_ambient,
            &self.coolant_pump_motor,
            &self.reservoir,
        ];

        sensors.iter().any(|i| i.is_err())
    }
}
