use core::time::Duration as CoreDuration;
use defmt::{info, warn};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, pubsub::PubSubChannel};
use embassy_time::{Duration, Ticker};
use hoshiguma_protocol::peripheral_controller::{
    event::Event,
    rpc::{Request, Response},
};
use static_cell::StaticCell;
use teeny_rpc::{client::Client, transport::embedded::EioTransport};

pub(crate) static TELEMETRY_MESSAGES: PubSubChannel<CriticalSectionRawMutex, Event, 64, 2, 1> =
    PubSubChannel::new();

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(super) async fn task(r: crate::TelemetryUartResources) {
    const TX_BUFFER_SIZE: usize = 32;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 256;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = hoshiguma_protocol::peripheral_controller::SERIAL_BAUD;

    let uart = BufferedUart::new(r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buf, rx_buf, config);

    // Setup RPC client
    let transport = EioTransport::<_, 512>::new(uart);
    let mut client = Client::<_, Request, Response>::new(transport, CoreDuration::from_millis(100));

    let tx = TELEMETRY_MESSAGES.publisher().unwrap();

    // Request events every 200ms
    let mut ticker = Ticker::every(Duration::from_millis(200));

    'telem_rx: loop {
        match client
            .call(Request::GetOldestEvent, CoreDuration::from_millis(50))
            .await
        {
            Ok(Response::GetOldestEvent(Some(event))) => {
                info!("Got event from controller: {:?}", event);
                tx.publish(event).await;

                // Immediately request further events
                ticker.reset();
                continue 'telem_rx;
            }
            Ok(Response::GetOldestEvent(None)) => {
                // Do nothing
            }
            Ok(_) => {
                warn!("Unexpected RPC response");
            }
            Err(e) => {
                warn!("RPC error: {}", e);
            }
        }

        ticker.next().await;
    }
}
