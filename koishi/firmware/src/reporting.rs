use crate::hal::Usart;

type Message = telemetry_protocols::Message<telemetry_protocols::koishi::Payload>;

fn report<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX>(
    serial: &mut Usart<USART, TX, RX>,
    msg: &Message,
) {
    let data = postcard::to_vec_cobs::<Message, 128>(&msg).unwrap();

    for i in data {
        serial.write_byte(i);
    }
    serial.flush();
}

pub(crate) fn boot<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX>(
    serial: &mut Usart<USART, TX, RX>,
) {
    let msg = Message {
        time: crate::hal::millis(),
        iteration_id: None,
        payload: telemetry_protocols::Payload::Boot(telemetry_protocols::Boot {
            name: "koishi".try_into().unwrap(),
            git_revision: git_version::git_version!().try_into().unwrap(),
        }),
    };

    report(serial, &msg);
}

pub(crate) fn panic<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX>(
    serial: &mut Usart<USART, TX, RX>,
    info: &core::panic::PanicInfo,
) {
    let msg = Message {
        time: crate::hal::millis(),
        iteration_id: None,
        payload: telemetry_protocols::Payload::Panic(info.into()),
    };

    report(serial, &msg);
}

pub(crate) fn status<USART: atmega_hal::usart::UsartOps<atmega_hal::Atmega, TX, RX>, TX, RX, T>(
    serial: &mut Usart<USART, TX, RX>,
    iteration_id: u32,
    status_payload: T,
) where
    telemetry_protocols::koishi::Payload: From<T>,
{
    let msg = Message {
        time: crate::hal::millis(),
        iteration_id: Some(iteration_id),
        payload: telemetry_protocols::Payload::Application(status_payload.into()),
    };

    report(serial, &msg);
}
