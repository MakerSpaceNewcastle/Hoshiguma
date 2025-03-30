use crate::CoolerCommunicationResources;
use defmt::{info, warn};
use embassy_futures::select::{select, Either};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART1,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use embassy_sync::{blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel};
use embassy_time::{Duration, Timer};
use hoshiguma_protocol::cooler::rpc::{Request, Response};
use static_cell::StaticCell;
use teeny_rpc::{client::Client, transport::embedded::EioTransport};

static FUCK: Channel<CriticalSectionRawMutex, (), 8> = Channel::new();

bind_interrupts!(struct Irqs {
    UART1_IRQ  => BufferedInterruptHandler<UART1>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: CoolerCommunicationResources) {
    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 256;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = hoshiguma_protocol::peripheral_controller::SERIAL_BAUD;

    let uart = BufferedUart::new(r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buf, rx_buf, config);

    // Setup RPC client
    let transport = EioTransport::new(uart);
    let mut client = Client::<_, Request, Response>::new(transport);

    const SHORT_EVENT_POLL: Duration = Duration::from_millis(50);
    const LONG_EVENT_POLL: Duration = Duration::from_millis(500);

    let mut delta = LONG_EVENT_POLL;

    let rx = FUCK.receiver();

    loop {
        match select(Timer::after(delta), rx.receive()).await {
            Either::First(_) => {
                match client
                    .call(
                        Request::GetOldestEvent,
                        core::time::Duration::from_millis(50),
                    )
                    .await
                {
                    Ok(Response::GetOldestEvent(Some(event))) => {
                        info!("Got event from cooler: {:?}", event);
                        // TODO
                        delta = SHORT_EVENT_POLL;
                    }
                    Ok(Response::GetOldestEvent(None)) => {
                        delta = LONG_EVENT_POLL;
                    }
                    Ok(_) => {
                        warn!("Unexpected RPC response");
                    }
                    Err(e) => {
                        warn!("RPC error: {}", e);
                    }
                }
            }
            Either::Second(cmd) => {
                todo!()
            }
        }
    }
}
