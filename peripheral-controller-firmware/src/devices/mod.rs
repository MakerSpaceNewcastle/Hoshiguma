pub(crate) mod air_assist_demand_detector;
pub(crate) mod air_assist_pump;
pub(crate) mod chassis_intrusion_detector;
pub(crate) mod cooler;
pub(crate) mod fume_extraction_fan;
pub(crate) mod fume_extraction_mode_switch;
pub(crate) mod laser_enable;
pub(crate) mod machine_enable;
pub(crate) mod machine_power_detector;
pub(crate) mod machine_run_detector;
pub(crate) mod status_lamp;
pub(crate) mod temperature_sensors;

pub(crate) trait TemperaturesExt {
    fn any_failed_sensors(&self) -> bool;
}
