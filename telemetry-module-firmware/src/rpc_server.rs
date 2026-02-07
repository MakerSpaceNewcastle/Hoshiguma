use crate::{
    TELEMETRY_TX,
    network::LINK_STATE,
    self_telemetry::{
        RPC_REQUEST_CLEARSTRINGS, RPC_REQUEST_DATAPOINT, RPC_REQUEST_DATAPOINT_FAIL_RENDER,
        RPC_REQUEST_DATAPOINT_FAIL_TOLINE, RPC_REQUEST_GETTIME, RPC_REQUEST_ISREADY,
        RPC_REQUEST_PUSHSTRING, RPC_REQUEST_STRINGSMETADATA, STRING_REGISTRY_LAST_MODIFIED,
        STRING_REGISTRY_SIZE,
    },
};
use core::{sync::atomic::Ordering, time::Duration as CoreDuration};
use defmt::{debug, warn};
use embassy_net::Stack;
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use embassy_time::Instant;
use hoshiguma_core::telemetry_module::{
    StringRegistry,
    rpc::{Request, Response},
};
use static_cell::StaticCell;
use teeny_rpc::{server::Server, transport::embedded::EioTransport};

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: crate::Rs485Uart1Resources, net_stack: Stack<'static>) {
    const TX_BUFFER_SIZE: usize = 32;
    static TX_BUFFER: StaticCell<[u8; TX_BUFFER_SIZE]> = StaticCell::new();
    let tx_buf = &mut TX_BUFFER.init([0; TX_BUFFER_SIZE])[..];

    const RX_BUFFER_SIZE: usize = 256;
    static RX_BUFFER: StaticCell<[u8; RX_BUFFER_SIZE]> = StaticCell::new();
    let rx_buf = &mut RX_BUFFER.init([0; RX_BUFFER_SIZE])[..];

    let mut config = UartConfig::default();
    config.baudrate = hoshiguma_core::telemetry_module::SERIAL_BAUD;

    let uart = BufferedUart::new(r.uart, r.tx_pin, r.rx_pin, Irqs, tx_buf, rx_buf, config);

    // Setup RPC server
    let transport = EioTransport::<_, 512>::new(uart);
    let mut server = Server::<_, Request, Response>::new(transport, CoreDuration::from_millis(100));

    // String storage for static strings
    let mut strings = StringRegistry::default();

    // Channel for sending formatted data point lines
    let telemetry_tx = TELEMETRY_TX.publisher().unwrap();

    loop {
        match server.wait_for_request(CoreDuration::from_secs(5)).await {
            Ok(Request::IsReady) => {
                let link_state = LINK_STATE.lock(|v| v.borrow().clone());
                let ready = link_state.dhcp4_config.is_some();

                if let Err(e) = server.send_response(Response::IsReady(ready)).await {
                    warn!("{}", e);
                }

                RPC_REQUEST_ISREADY.fetch_add(1, Ordering::Relaxed);
            }
            Ok(Request::GetWallTime) => {
                let result = crate::network::time::get_unix_timestamp(net_stack).await;

                if let Err(e) = server.send_response(Response::GetWallTime(result)).await {
                    warn!("{}", e);
                }

                RPC_REQUEST_GETTIME.fetch_add(1, Ordering::Relaxed);
            }
            Ok(Request::GetStringRegistryMetadata) => {
                if let Err(e) = server
                    .send_response(Response::GetStringRegistryMetadata(strings.metadata()))
                    .await
                {
                    warn!("{}", e);
                }

                RPC_REQUEST_STRINGSMETADATA.fetch_add(1, Ordering::Relaxed);
            }
            Ok(Request::ClearStringRegistry) => {
                strings.clear();

                if let Err(e) = server.send_response(Response::ClearStringRegistry).await {
                    warn!("{}", e);
                }

                STRING_REGISTRY_SIZE.store(strings.len(), Ordering::Relaxed);
                STRING_REGISTRY_LAST_MODIFIED.store(Instant::now().as_millis(), Ordering::Relaxed);

                RPC_REQUEST_CLEARSTRINGS.fetch_add(1, Ordering::Relaxed);
            }
            Ok(Request::PushStringToRegistry(str)) => {
                let result = strings.push(str);

                if let Err(e) = server
                    .send_response(Response::PushStringToRegistry(result))
                    .await
                {
                    warn!("{}", e);
                }

                STRING_REGISTRY_SIZE.store(strings.len(), Ordering::Relaxed);
                STRING_REGISTRY_LAST_MODIFIED.store(Instant::now().as_millis(), Ordering::Relaxed);

                RPC_REQUEST_PUSHSTRING.fetch_add(1, Ordering::Relaxed);
            }
            Ok(Request::SendTelemetryDataPoint(data_point)) => {
                debug!("{}", data_point);
                let result = match data_point.to_rendered_data_point(&strings) {
                    Ok(data_point) => {
                        debug!("{}", data_point);
                        match data_point.to_influx_line_string() {
                            Ok(line) => {
                                telemetry_tx.publish_immediate(line);
                                Ok(())
                            }
                            Err(e) => {
                                warn!("Failed to generate influx line string: {}", e);
                                RPC_REQUEST_DATAPOINT_FAIL_TOLINE.fetch_add(1, Ordering::Relaxed);
                                Err(())
                            }
                        }
                    }
                    Err(e) => {
                        warn!("Failed to render data point: {}", e);
                        RPC_REQUEST_DATAPOINT_FAIL_RENDER.fetch_add(1, Ordering::Relaxed);
                        Err(())
                    }
                };

                if let Err(e) = server
                    .send_response(Response::SendTelemetryDataPoint(result))
                    .await
                {
                    warn!("{}", e);
                }

                RPC_REQUEST_DATAPOINT.fetch_add(1, Ordering::Relaxed);
            }
            Err(teeny_rpc::Error::Timeout) => {
                debug!("Timeout when waiting for request");
            }
            Err(e) => {
                warn!("Server failed waiting for request: {}", e);
            }
        }
    }
}
