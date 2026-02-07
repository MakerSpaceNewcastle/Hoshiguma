use crate::telemetry::{AsTelemetry, TelemetryStrValue};

pub mod rpc;

pub const SERIAL_BAUD: u32 = 115_200;

// If neither the capacity or string length had to be changed, then the firmware
// on the telemetry module does not need to be reflashed when modifying
// telemetry on the peripheral controller.
pub const STRING_REGISTRY_CAPACITY: usize = 80;
pub const STRING_REGISTRY_MAX_STRING_LENGTH: usize = 60;

pub type StringRegistry = crate::string_registry::StringRegistry<
    STRING_REGISTRY_CAPACITY,
    STRING_REGISTRY_MAX_STRING_LENGTH,
>;

pub fn build_string_registry() -> crate::string_registry::Result<StringRegistry> {
    let mut r = StringRegistry::from_slice(&[
        // Measurements
        "peripheral_controller_git_revision",
        "peripheral_controller_boot_reason",
        "peripheral_controller_uptime",
        "peripheral_controller_data_point_template_errors",
        "peripheral_controller_data_points_discarded",
        // Fields
        "value",
        // Values
        crate::types::BootReason::Normal.telemetry_str(),
        crate::types::BootReason::WatchdogTimeout.telemetry_str(),
        crate::types::BootReason::WatchdogForced.telemetry_str(),
    ])
    .unwrap();

    r.extend_from_str_slice(&crate::types::Monitors::strings())?;
    r.extend_from_str_slice(&crate::types::MachineOperationLockout::strings())?;
    r.extend_from_str_slice(&crate::types::CoolingEnable::strings())?;
    r.extend_from_str_slice(&crate::types::CoolingDemand::strings())?;
    r.extend_from_str_slice(&crate::types::AirAssistDemand::strings())?;
    r.extend_from_str_slice(&crate::types::AirAssistPump::strings())?;
    r.extend_from_str_slice(&crate::types::FumeExtractionMode::strings())?;
    r.extend_from_str_slice(&crate::types::FumeExtractionFan::strings())?;
    r.extend_from_str_slice(&crate::types::LaserEnable::strings())?;
    r.extend_from_str_slice(&crate::types::MachineEnable::strings())?;
    r.extend_from_str_slice(&crate::types::ChassisIntrusion::strings())?;
    r.extend_from_str_slice(&crate::types::MachinePower::strings())?;
    r.extend_from_str_slice(&crate::types::MachineRun::strings())?;
    r.extend_from_str_slice(&crate::types::MachineTemperatures::strings())?;
    r.extend_from_str_slice(&crate::accessories::cooler::types::Temperatures::strings())?;
    r.extend_from_str_slice(&crate::accessories::cooler::types::CoolantPumpState::strings())?;
    r.extend_from_str_slice(&crate::accessories::cooler::types::CompressorState::strings())?;
    r.extend_from_str_slice(&crate::accessories::cooler::types::RadiatorFanState::strings())?;
    r.extend_from_str_slice(&crate::accessories::cooler::types::CoolantReservoirLevel::strings())?;
    r.extend_from_str_slice(&crate::accessories::cooler::types::CoolantFlow::strings())?;
    r.extend_from_str_slice(
        &crate::accessories::extraction_airflow_sensor::types::Measurement::strings(),
    )?;

    Ok(r)
}

#[cfg(test)]
mod test {
    use super::*;
    use core::cmp::max;

    #[test]
    fn test_build_string_registry() {
        let sr = build_string_registry().unwrap();

        let mut max_string_len = 0;
        for idx in 0..sr.len() {
            let s = sr.get_string(idx).unwrap();
            println!("{idx} = {:?}", s);
            max_string_len = max(max_string_len, s.len());
        }

        assert!(sr.len() < STRING_REGISTRY_CAPACITY);
        assert!(max_string_len < STRING_REGISTRY_MAX_STRING_LENGTH);
    }
}
