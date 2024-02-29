pub(crate) mod coolant_level;
pub(crate) mod frequency_counter;
pub(crate) mod temperature;

pub(crate) trait SensorReadAndUpdate {
    fn read(&mut self);
}
