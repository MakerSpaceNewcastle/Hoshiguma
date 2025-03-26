use crate::ControlCommunicationResources;
use defmt::warn;
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart},
};
use hoshiguma_protocol::cooler::rpc::{Request, Response};
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
    config.baudrate = 115_200;

    let uart = BufferedUart::new(r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buffer, rx_buffer, config);

    let transport = teeny_rpc::transport::embedded::EioTransport::new(uart);
    let mut server = teeny_rpc::server::Server::<_, Request, Response>::new(transport);

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
