use crate::TelemetryResources;
use core::time::Duration as CoreDuration;
use defmt::warn;
use embassy_futures::select::{select, Either};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::Instant;
use hoshiguma_protocol::{
    event_queue::EventQueue,
    peripheral_controller::{
        event::{Event, EventKind},
        rpc::{Request, Response},
    },
};
use static_cell::StaticCell;
use teeny_rpc::{server::Server, transport::embedded::EioTransport};

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(super) async fn task(r: TelemetryResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("telemetry").await;

    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buffer = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 32;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buffer = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = hoshiguma_protocol::peripheral_controller::SERIAL_BAUD;

    let uart = BufferedUart::new(
        r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buffer, rx_buffer, config,
    );

    // Setup RPC server
    let transport = EioTransport::<_, 512>::new(uart);
    let mut server = Server::<_, Request, Response>::new(transport, CoreDuration::from_millis(100));

    // Queue to hold events before they are requested
    let mut event_queue = EventQueue::<_, 32>::default();

    // Report boot
    queue_telemetry_event(EventKind::Boot(crate::system_information())).await;

    loop {
        match select(
            server.wait_for_request(CoreDuration::from_secs(5)),
            TELEMETRY_MESSAGES.receive(),
        )
        .await
        {
            Either::First(Ok(request)) => match request {
                Request::Ping(i) => {
                    if let Err(e) = server.send_response(Response::Ping(i)).await {
                        warn!("Server failed sending response: {}", e);
                    }
                }
                Request::GetSystemInformation => {
                    if let Err(e) = server
                        .send_response(Response::GetSystemInformation(crate::system_information()))
                        .await
                    {
                        warn!("Server failed sending response: {}", e);
                    }
                }
                Request::GetEventCount => {
                    if let Err(e) = server
                        .send_response(Response::GetEventCount(event_queue.len()))
                        .await
                    {
                        warn!("Server failed sending response: {}", e);
                    }
                }
                Request::GetEventStatistics => {
                    if let Err(e) = server
                        .send_response(Response::GetEventStatistics(event_queue.statistics()))
                        .await
                    {
                        warn!("Server failed sending response: {}", e);
                    }
                }
                Request::GetOldestEvent => {
                    let transaction = event_queue.ret_request();
                    let event = event_queue.ret_get(&transaction);

                    match server.send_response(Response::GetOldestEvent(event)).await {
                        Ok(_) => {
                            event_queue.ret_commit(transaction);
                        }
                        Err(e) => {
                            warn!("Server failed sending response: {}", e);
                        }
                    }
                }
            },
            Either::First(Err(e)) => {
                warn!("Server failed waiting for request: {}", e);
            }
            Either::Second(event) => {
                event_queue.push(event);
            }
        }
    }
}

static TELEMETRY_MESSAGES: Channel<CriticalSectionRawMutex, Event, 8> = Channel::new();

/// Queues a telemetry event to be retrieved via RPC.
///
/// # Arguments
///
/// * `event` - The kind of event to be queued.
pub(crate) async fn queue_telemetry_event(event: EventKind) {
    let event = Event {
        timestamp_milliseconds: Instant::now().as_millis(),
        kind: event,
    };
    TELEMETRY_MESSAGES.send(event).await;
}
