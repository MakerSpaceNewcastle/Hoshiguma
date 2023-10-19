use super::SensorReadAndUpdate;
use esp_idf_hal::gpio::PinDriver;
use esp_idf_sys as _;

pub(crate) struct CoolantLevelSensor<
    'a,
    U: esp_idf_hal::gpio::Pin,
    L: esp_idf_hal::gpio::Pin,
    B: esp_idf_hal::gpio::InputMode,
> {
    upper: PinDriver<'a, U, B>,
    lower: PinDriver<'a, L, B>,
}

impl<'a, U, L, B> CoolantLevelSensor<'a, U, L, B>
where
    U: esp_idf_hal::gpio::Pin,
    L: esp_idf_hal::gpio::Pin,
    B: esp_idf_hal::gpio::InputMode,
{
    pub fn new(upper: PinDriver<'a, U, B>, lower: PinDriver<'a, L, B>) -> Self {
        Self { upper, lower }
    }
}

impl<'a, U: esp_idf_hal::gpio::Pin, L: esp_idf_hal::gpio::Pin, B: esp_idf_hal::gpio::InputMode>
    SensorReadAndUpdate for CoolantLevelSensor<'a, U, L, B>
{
    fn read(&mut self) {
        let _upper = self.upper.is_high();
        let _lower = self.lower.is_high();

        // TODO
    }
}
