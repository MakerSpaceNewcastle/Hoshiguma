use crate::ControlCommunicationResources;
use defmt::warn;
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Instant;
use hoshiguma_protocol::{
    cooler::{
        event::{Event, EventKind},
        rpc::{Request, Response},
        SERIAL_BAUD,
    },
    event_queue::EventQueue,
};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: ControlCommunicationResources) {
    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buffer = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 256;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buffer = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = embassy_rp::uart::Config::default();
    config.baudrate = SERIAL_BAUD;

    let uart = BufferedUart::new(
        r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buffer, rx_buffer, config,
    );

    let transport = teeny_rpc::transport::embedded::EioTransport::new(uart);
    let mut server = teeny_rpc::server::Server::<_, Request, Response>::new(transport);

    // Queue to hold events before they are requested
    let mut event_queue = EventQueue::<_, 32>::default();

    // Report boot
    report_event(EventKind::Boot(crate::system_information())).await;

    loop {
        match server
            .wait_for_request(core::time::Duration::from_secs(5))
            .await
        {
            Ok(request) => {
                // let response = match request {
                //     Request::Ping(i) => Response::Ping(i),
                // };
                // if let Err(e) = server.send_response(response).await {
                //     warn!("Server failed sending response: {}", e);
                // }
            }
            Err(e) => {
                warn!("Server failed waiting for request: {}", e);
            }
        }
    }
}

static NEW_EVENT: Channel<CriticalSectionRawMutex, Event, 8> = Channel::new();

/// Queues a telemetry event to be retrieved via RPC.
///
/// # Arguments
///
/// * `event` - The kind of event to be queued.
pub(crate) async fn report_event(event: EventKind) {
    let event = Event {
        timestamp_milliseconds: Instant::now().as_millis(),
        kind: event,
    };
    NEW_EVENT.send(event).await;
}
