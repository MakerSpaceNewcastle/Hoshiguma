use super::{
    air_assist::{AirAssistDemandDetector, AIR_ASSIST_DEMAND_CHANGED},
    chassis_intrusion_detector::{ChassisIntrusionDetector, CHASSIS_INTRUSION_CHANGED},
    coolant_resevoir_level_sensor::{CoolantResevoirLevelSensor, COOLANT_RESEVOIR_LEVEL_CHANGED},
    fume_extraction_mode_switch::{FumeExtractionModeSwitch, FUME_EXTRACTION_MODE_CHANGED},
    machine_run_detector::{MachineRunDetector, MACHINE_RUNNING_CHANGED},
};
#[cfg(feature = "telemetry")]
use crate::telemetry::queue_telemetry_message;
use crate::{
    AirAssistDemandDetectResources, ChassisIntrusionDetectResources,
    CoolantResevoirLevelSensorResources, FumeExtractionModeSwitchResources,
    MachineRunDetectResources,
};
use embassy_time::{Duration, Ticker};
#[cfg(feature = "telemetry")]
use hoshiguma_telemetry_protocol::payload::{observation::ObservationPayload, Payload};

#[embassy_executor::task]
pub(crate) async fn task(
    chassis_intrusion_detector_resources: ChassisIntrusionDetectResources,
    air_assist_demand_detector_resources: AirAssistDemandDetectResources,
    machine_run_detector_resources: MachineRunDetectResources,
    fume_extraction_mode_switch_resources: FumeExtractionModeSwitchResources,
    coolant_resevoir_level_sensor_resources: CoolantResevoirLevelSensorResources,
) {
    let mut chassis_intrusion_detector: ChassisIntrusionDetector =
        chassis_intrusion_detector_resources.into();
    let mut air_assist_demand_detector: AirAssistDemandDetector =
        air_assist_demand_detector_resources.into();
    let mut machine_run_detector: MachineRunDetector = machine_run_detector_resources.into();
    let mut fume_extraction_mode_switch: FumeExtractionModeSwitch =
        fume_extraction_mode_switch_resources.into();
    let mut coolant_resevoir_level_sensor: CoolantResevoirLevelSensor =
        coolant_resevoir_level_sensor_resources.into();

    let mut ticker = Ticker::every(Duration::from_micros(10));

    let chassis_intrusion_tx = CHASSIS_INTRUSION_CHANGED.sender();
    let air_assist_tx = AIR_ASSIST_DEMAND_CHANGED.sender();
    let machine_run_tx = MACHINE_RUNNING_CHANGED.sender();
    let fume_extraction_mode_tx = FUME_EXTRACTION_MODE_CHANGED.sender();
    let coolant_resevoir_level_tx = COOLANT_RESEVOIR_LEVEL_CHANGED.sender();

    loop {
        ticker.next().await;

        if let Some(state) = chassis_intrusion_detector.update() {
            #[cfg(feature = "telemetry")]
            queue_telemetry_message(Payload::Observation(ObservationPayload::ChassisIntrusion(
                (&state).into(),
            )))
            .await;

            chassis_intrusion_tx.send(state);
        }

        if let Some(state) = air_assist_demand_detector.update() {
            #[cfg(feature = "telemetry")]
            queue_telemetry_message(Payload::Observation(ObservationPayload::AirAssistDemand(
                (&state).into(),
            )))
            .await;

            air_assist_tx.send(state);
        }

        if let Some(state) = machine_run_detector.update() {
            #[cfg(feature = "telemetry")]
            queue_telemetry_message(Payload::Observation(ObservationPayload::MachineRun(
                (&state).into(),
            )))
            .await;

            machine_run_tx.send(state);
        }

        if let Some(state) = fume_extraction_mode_switch.update() {
            #[cfg(feature = "telemetry")]
            queue_telemetry_message(Payload::Observation(
                ObservationPayload::FumeExtractionMode((&state).into()),
            ))
            .await;

            fume_extraction_mode_tx.send(state);
        }

        if let Some(state) = coolant_resevoir_level_sensor.update() {
            #[cfg(feature = "telemetry")]
            queue_telemetry_message(Payload::Observation(
                ObservationPayload::CoolantResevoirLevel((&state).into()),
            ))
            .await;

            coolant_resevoir_level_tx.send(state);
        }
    }
}
