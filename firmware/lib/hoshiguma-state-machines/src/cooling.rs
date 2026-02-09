use defmt::debug;
use hoshiguma_api::{
    AcBusPower, TemperatureReading, TemperatureSensor, TemperatureSensorReading,
    cooler::{CompressorState, CoolantPumpState, RadiatorFanState},
};
use hoshiguma_common::changed::ObservedValue;

crate::state_machine!(InputMessage, OutputMessage, State, 4);

pub enum InputMessage {
    AcBusPower(AcBusPower),
    Temperature(TemperatureSensorReading),
}

#[derive(Debug, PartialEq)]
pub enum OutputMessage {
    CoolantPump(CoolantPumpState),
    RadiatorFan(RadiatorFanState),
    Compressor(CompressorState),
}

pub struct State {
    ac_bus_power: AcBusPower,
    reservoir_temperature: TemperatureReading,

    output_coolant_pump: ObservedValue<CoolantPumpState>,
    output_radiator_fan: ObservedValue<RadiatorFanState>,
    output_compressor: ObservedValue<CompressorState>,
}

impl Default for State {
    fn default() -> Self {
        Self {
            ac_bus_power: AcBusPower::Off,
            reservoir_temperature: Err(()),

            output_coolant_pump: ObservedValue::default(),
            output_radiator_fan: ObservedValue::default(),
            output_compressor: ObservedValue::default(),
        }
    }
}

const UPPER_TEMPERATURE: f32 = 17.5;
const LOWER_TEMPERATURE: f32 = 17.0;

impl<'a> crate::StateMachineRun for StateMachineRunner<'a> {
    async fn run(&mut self) -> ! {
        loop {
            match self.input_channel.receive().await {
                InputMessage::AcBusPower(state) => {
                    self.state.ac_bus_power = state;
                }
                InputMessage::Temperature(reading) => {
                    if reading.sensor == TemperatureSensor::CoolantReservoir {
                        self.state.reservoir_temperature = reading.reading;
                    }
                }
            }

            let (pump, fan, compressor) = if self.state.ac_bus_power == AcBusPower::On {
                let compressor = if let Ok(temperature) = self.state.reservoir_temperature {
                    if temperature > UPPER_TEMPERATURE {
                        Some(CompressorState::Run)
                    } else if temperature < LOWER_TEMPERATURE {
                        Some(CompressorState::Idle)
                    } else {
                        None
                    }
                } else {
                    None
                };

                // Keep the old setting if in the hysteresis band or if the temperature is unavailable, otherwise update to the new demand.
                let compressor = compressor.unwrap_or_else(|| {
                    // If there is no previous setting then default to off.
                    (*self.state.output_compressor).unwrap_or(CompressorState::Idle)
                });

                // Coolant pump and radiator fan always run when the machine is on
                (CoolantPumpState::Run, RadiatorFanState::Run, compressor)
            } else {
                // Everything off if the AC bus power is off
                // Technically the cooler can run if the machine AC bus is off, however this also allows the cooler to be turned off when the E-stop is pressed
                (
                    CoolantPumpState::Idle,
                    RadiatorFanState::Idle,
                    CompressorState::Idle,
                )
            };

            debug!(
                "coolant pump {}, radiator fan {}, compressor {}",
                pump, fan, compressor
            );

            self.state
                .output_coolant_pump
                .update_and_async(pump, async |v| {
                    self.output_channel
                        .send(OutputMessage::CoolantPump(v))
                        .await;
                })
                .await;

            self.state
                .output_radiator_fan
                .update_and_async(fan, async |v| {
                    self.output_channel
                        .send(OutputMessage::RadiatorFan(v))
                        .await;
                })
                .await;

            self.state
                .output_compressor
                .update_and_async(compressor, async |v| {
                    self.output_channel.send(OutputMessage::Compressor(v)).await;
                })
                .await;
        }
    }
}
