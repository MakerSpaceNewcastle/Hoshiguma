use crate::CommunicationResources;
use core::time::Duration as CoreDuration;
use defmt::{debug, warn};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use hoshiguma_protocol::accessories::{
    SERIAL_BAUD,
    extraction_airflow_sensor::rpc::{Request as SensorRequest, Response as SensorResponse},
    rpc::{Request, Response},
};
use static_cell::StaticCell;
use teeny_rpc::{server::Server, transport::embedded::EioTransport};

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: CommunicationResources, mut airflow_sensor: crate::sdp810::Sdp810) {
    const TX_BUFFER_SIZE: usize = 256;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buffer = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 256;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buffer = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = SERIAL_BAUD;

    let uart = BufferedUart::new(
        r.uart, r.tx_pin, r.rx_pin, Irqs, tx_buffer, rx_buffer, config,
    );

    let transport = EioTransport::<_, 512>::new(uart);
    let mut server = Server::<_, Request, Response>::new(transport, CoreDuration::from_millis(100));

    loop {
        match server.wait_for_request(CoreDuration::from_secs(5)).await {
            Ok(Request::ExtractionAirflowSensor(request)) => {
                let response = match request {
                    SensorRequest::Ping(i) => SensorResponse::Ping(i),
                    SensorRequest::GetSystemInformation => {
                        SensorResponse::GetSystemInformation(crate::system_information())
                    }
                    SensorRequest::GetMeasurement => {
                        SensorResponse::GetMeasurement(airflow_sensor.get_measurement().await)
                    }
                };

                if let Err(e) = server
                    .send_response(Response::ExtractionAirflowSensor(response))
                    .await
                {
                    warn!("Server failed sending response: {}", e);
                }
            }
            Ok(_) => {
                debug!("Got request that was not for us");
                server.ignore_active_request().expect("There should be an active request here, as we are opting to ignore it based on it's payload");
            }
            Err(e) => {
                warn!("Server failed waiting for request: {}", e);
            }
        }
    }
}
