use super::Message;
use crate::hal::Usart;

pub(super) fn report<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX>(
    serial: &mut Usart<USART, TX, RX>,
    msg: &Message,
) {
    let data = postcard::to_vec_cobs::<Message, 128>(&msg).unwrap();

    for i in data {
        serial.write_byte(i);
    }
    serial.flush();
}
