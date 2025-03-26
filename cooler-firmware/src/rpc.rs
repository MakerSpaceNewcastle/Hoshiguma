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
    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];

    let mut config = embassy_rp::uart::Config::default();
    config.baudrate = 115_200;

    let uart = BufferedUart::new(r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buf, rx_buf, config);

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
