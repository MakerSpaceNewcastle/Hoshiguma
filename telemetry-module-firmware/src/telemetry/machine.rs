use crate::{
    metric::{Metric, MetricKind},
    network::telemetry_tx::METRIC_TX,
};
use core::{sync::atomic::Ordering, time::Duration as CoreDuration};
use defmt::{info, warn};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use embassy_time::Timer;
use hoshiguma_protocol::peripheral_controller::rpc::{Request, Response};
use portable_atomic::AtomicU64;
use static_cell::StaticCell;
use teeny_rpc::{client::Client, transport::embedded::EioTransport};

pub(crate) static TELEMETRY_RX_SUCCESS: AtomicU64 = AtomicU64::new(0);
pub(crate) static TELEMETRY_RX_FAIL: AtomicU64 = AtomicU64::new(0);

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: crate::TelemetryUartResources) {
    #[cfg(feature = "trace")]
    crate::trace::name_task("machine telemetry").await;

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

    let metric_tx = METRIC_TX.publisher().unwrap();

    'telem_rx: loop {
        match client
            .call(Request::GetOldestEvent, CoreDuration::from_millis(50))
            .await
        {
            Ok(Response::GetOldestEvent(Some(event))) => {
                info!("Got event from controller: {:?}", event);
                TELEMETRY_RX_SUCCESS.add(1, Ordering::Relaxed);
                metric_tx
                    .publish(Metric::new(
                        crate::network::time::wall_time(),
                        MetricKind::PeripheralControllerEvent(event.kind),
                    ))
                    .await;

                // Immediately request further events
                continue 'telem_rx;
            }
            Ok(Response::GetOldestEvent(None)) => {
                // Do nothing
            }
            Ok(_) => {
                warn!("Unexpected RPC response");
                TELEMETRY_RX_FAIL.add(1, Ordering::Relaxed);
            }
            Err(e) => {
                warn!("RPC error: {}", e);
                TELEMETRY_RX_FAIL.add(1, Ordering::Relaxed);
            }
        }

        // Request events every second
        Timer::after_secs(1).await;
    }
}
