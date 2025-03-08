use defmt::{info, trace, warn};
use embassy_rp::{
    peripherals::UART0,
    uart::{Config as UartConfig, InterruptHandler, UartRx},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use heapless::Vec;
use hoshiguma_protocol::{serial::TELEMETRY_BAUD, Message};

pub(crate) static TELEMETRY_MESSAGES: PubSubChannel<CriticalSectionRawMutex, Message, 64, 2, 1> =
    PubSubChannel::new();

embassy_rp::bind_interrupts!(pub(super) struct Irqs {
    UART0_IRQ => InterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(super) async fn task(r: crate::TelemetryUartResources) {
    let mut rx = {
        let mut config = UartConfig::default();
        config.baudrate = TELEMETRY_BAUD;
        UartRx::new(r.uart, r.rx_pin, Irqs, r.dma_ch, config)
    };
    let tx = TELEMETRY_MESSAGES.publisher().unwrap();

    let mut buffer: Vec<u8, 100> = Vec::new();

    loop {
        let mut b = [0u8];

        match rx.read(&mut b).await {
            Ok(_) => {
                buffer.extend(b);

                if buffer.last() == Some(&0u8) {
                    match postcard::from_bytes_cobs::<Message>(buffer.as_mut_slice()) {
                        Ok(msg) => {
                            info!("Received message: {}", msg);
                            tx.publish(msg).await;
                        }
                        Err(_) => warn!(
                            "Failed to decode message with {} bytes in buffer",
                            buffer.len(),
                        ),
                    }

                    buffer.clear();
                }
            }
            Err(_) => trace!("UART read fail"),
        }
    }
}
