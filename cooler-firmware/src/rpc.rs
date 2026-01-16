use crate::{ControlCommunicationResources, machine::Machine};
use core::time::Duration as CoreDuration;
use defmt::{debug, warn};
use embassy_futures::select::{Either, select};
use embassy_time::{Duration, Instant, Ticker};
use hoshiguma_protocol::accessories::{
    SERIAL_BAUD,
    cooler::rpc::{Request as CoolerRequest, Response as CoolerResponse},
    rpc::{Request, Response},
};
use pico_plc_bsp::embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use static_cell::StaticCell;
use teeny_rpc::{server::Server, transport::embedded::EioTransport};

bind_interrupts!(struct Irqs {
    UART0_IRQ  => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::task]
pub(crate) async fn task(r: ControlCommunicationResources, mut machine: Machine) {
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

    let mut watchdog = CommunicationWatchdog::new(Duration::from_secs(5));
    let mut watchdog_check_tick = Ticker::every(Duration::from_secs(1));

    loop {
        match select(
            server.wait_for_request(CoreDuration::from_secs(5)),
            watchdog_check_tick.next(),
        )
        .await
        {
            Either::First(Ok(Request::Cooler(request))) => {
                watchdog.feed();

                let response = match request {
                    CoolerRequest::Ping(i) => CoolerResponse::Ping(i),
                    CoolerRequest::GetSystemInformation => {
                        CoolerResponse::GetSystemInformation(crate::system_information())
                    }
                    CoolerRequest::GetState => CoolerResponse::GetState(machine.state().await),
                    CoolerRequest::SetRadiatorFan(setting) => {
                        machine.radiator_fan.set(setting);
                        CoolerResponse::SetRadiatorFan
                    }
                    CoolerRequest::SetCompressor(setting) => {
                        machine.compressor.set(setting);
                        CoolerResponse::SetCompressor
                    }
                    CoolerRequest::SetCoolantPump(setting) => {
                        machine.coolant_pump.set(setting);
                        CoolerResponse::SetCoolantPump
                    }
                };

                if let Err(e) = server.send_response(Response::Cooler(response)).await {
                    warn!("Server failed sending response: {}", e);
                }
            }
            Either::First(Ok(_)) => {
                debug!("Got request that was not for us");
                server.ignore_active_request().expect("There should be an active request here, as we are opting to ignore it based on it's payload");
            }
            Either::First(Err(e)) => {
                warn!("Server failed waiting for request: {}", e);
            }
            Either::Second(_) => {
                if watchdog.check() == CommunicationWatchdogState::Triggered {
                    warn!("Turning off cooling due to communication watchdog");
                    machine.set_off();
                }
            }
        }
    }
}

/// The `CommunicationWatchdog` is used to monitor communication and trigger an action if a timeout
/// occurs.
struct CommunicationWatchdog {
    /// The duration after which the watchdog will trigger if no communication is detected.
    timeout: Duration,

    /// The last instant when communication was detected.
    last: Instant,

    /// A boolean indicating whether the watchdog has been triggered.
    triggered: bool,
}

impl CommunicationWatchdog {
    fn new(timeout: Duration) -> Self {
        Self {
            timeout,
            last: Instant::now(),
            triggered: false,
        }
    }

    fn check(&mut self) -> CommunicationWatchdogState {
        let elapsed = Instant::now() - self.last;
        if elapsed >= self.timeout {
            if self.triggered {
                CommunicationWatchdogState::PreviouslyTriggered
            } else {
                self.triggered = true;
                CommunicationWatchdogState::Triggered
            }
        } else {
            CommunicationWatchdogState::Ok
        }
    }

    fn feed(&mut self) {
        self.last = Instant::now();
        self.triggered = false;
    }
}

#[derive(PartialEq, Eq)]
enum CommunicationWatchdogState {
    Ok,
    Triggered,
    PreviouslyTriggered,
}
