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
    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 32;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = hoshiguma_protocol::peripheral_controller::SERIAL_BAUD;

    let uart = BufferedUart::new(r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buf, rx_buf, config);

    // Setup RPC client
    let transport = EioTransport::new(uart);
    let mut client = Client::<_, Request, Response>::new(transport);

    let tx = TELEMETRY_MESSAGES.publisher().unwrap();

    let mut ticker = Ticker::every(Duration::from_millis(200));

    loop {
        ticker.next().await;

        match client
            .call(
                Request::GetOldestEvents(8),
                core::time::Duration::from_millis(50),
            )
            .await
        {
            Ok(Response::GetOldestEvents(events)) => {
                info!("Got {} events from controller", events.len());
                for event in events {
                    tx.publish(event).await;
                }
            }
            Ok(_) => {
                warn!("Unexpected RPC response");
            }
            Err(e) => {
                warn!("RPC error: {}", e);
            }
        }
    }
}
