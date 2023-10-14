#[cfg(feature = "reporting_postcard")]
mod postcard;
mod protocol;

use self::protocol::{BootPayload, Message, PanicPayload, Payload};
use crate::hal::Usart;

fn report<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX>(
    serial: &mut Usart<USART, TX, RX>,
    msg: &Message,
) {
    #[cfg(feature = "reporting_postcard")]
    self::postcard::report(serial, &msg);
}

pub(crate) fn boot<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX>(
    serial: &mut Usart<USART, TX, RX>,
) {
    report(
        serial,
        &Message::new(None, Payload::Boot(BootPayload::default())),
    );
}

pub(crate) fn panic<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX>(
    serial: &mut Usart<USART, TX, RX>,
    info: &core::panic::PanicInfo,
) {
    report(
        serial,
        &Message::new(None, Payload::Panic(PanicPayload::from(info))),
    );
}

pub(crate) fn status<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX, T>(
    serial: &mut Usart<USART, TX, RX>,
    iteration_id: u32,
    status_payload: T,
) where
    Payload: From<T>,
{
    let msg = Message::new(Some(iteration_id), status_payload.into());
    report(serial, &msg);
}
