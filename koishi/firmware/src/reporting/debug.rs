use super::Message;
use crate::hal::Usart;
use atmega_hal::prelude::*;

pub(super) fn report<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX>(
    serial: &mut Usart<USART, TX, RX>,
    msg: &Message,
) {
    ufmt::uwriteln!(serial, "{:#?}", msg).void_unwrap();
}
