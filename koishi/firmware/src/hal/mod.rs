mod pins;
pub(crate) use self::pins::Pins;

mod time;
pub(crate) use self::time::{millis, millis_init, Delay, TimeMillis};

pub(crate) type Clock = avr_hal_generic::clock::MHz8;

#[allow(dead_code)]
pub(crate) type Usart<USART, RX, TX> = atmega_hal::usart::Usart<USART, RX, TX, Clock>;

#[macro_export]
macro_rules! serial {
    ($p:expr, $pins:expr, $baud:expr) => {
        $crate::hal::Usart::new(
            $p.USART0,
            $pins.d0,
            $pins.d1.into_output(),
            atmega_hal::usart::BaudrateArduinoExt::into_baudrate($baud),
        )
    };
}
