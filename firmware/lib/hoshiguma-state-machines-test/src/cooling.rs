use crate::assert_queue_empty;
use embassy_sync::channel::Channel;
use embassy_time::Duration;
use hoshiguma_api::{
    AcBusPower, TemperatureSensor, TemperatureSensorReading,
    cooler::{CompressorState, CoolantPumpState, RadiatorFanState},
};
use hoshiguma_state_machines::cooling::{InputMessage, OutputMessage};

pub(super) async fn test_basic() {
    let input_channel = Channel::new();
    let output_channel = Channel::new();

    let (runner, mut communicator) =
        hoshiguma_state_machines::cooling::new(&input_channel, &output_channel);

    crate::run_test(Duration::from_secs(10), runner, async move || {
        // Machine starts off - first update emits initial state for all three outputs.
        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::Off))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantPump(CoolantPumpState::Idle)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RadiatorFan(RadiatorFanState::Idle)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Compressor(CompressorState::Idle)
        );

        // Turn the machine on with no temperature reading available.
        // Pump and fan start running; compressor has no temperature to act on so it
        // falls back to its previous state (idle) and therefore does not emit.
        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::On))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantPump(CoolantPumpState::Run)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RadiatorFan(RadiatorFanState::Run)
        );
        assert_queue_empty!(communicator);

        // Temperature updates for sensors other than CoolantReservoir are ignored.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantFlowAtTube,
                reading: Ok(20.0),
            }))
            .await;
        assert_queue_empty!(communicator);

        // Reservoir temperature above the upper threshold (17.5°C): compressor turns on.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantReservoir,
                reading: Ok(20.0),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Compressor(CompressorState::Run)
        );

        // Temperature falls into the hysteresis band (17.0°C–17.5°C).
        // The compressor retains its current state (running) and no output is emitted.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantReservoir,
                reading: Ok(17.2),
            }))
            .await;
        assert_queue_empty!(communicator);

        // Temperature falls below the lower threshold (17.0°C): compressor idles.
        communicator
            .send_input(InputMessage::Temperature(TemperatureSensorReading {
                sensor: TemperatureSensor::CoolantReservoir,
                reading: Ok(16.5),
            }))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::Compressor(CompressorState::Idle)
        );

        // Turn the machine off. Pump and fan go idle; compressor is already idle so
        // no additional output is emitted for it.
        communicator
            .send_input(InputMessage::AcBusPower(AcBusPower::Off))
            .await;
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::CoolantPump(CoolantPumpState::Idle)
        );
        assert_eq!(
            communicator.receive_output().await,
            OutputMessage::RadiatorFan(RadiatorFanState::Idle)
        );

        assert_queue_empty!(communicator);
    })
    .await;
}
