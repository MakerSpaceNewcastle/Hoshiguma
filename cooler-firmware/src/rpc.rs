use crate::{
    devices::{
        compressor::COMPRESSOR, coolant_pump::COOLANT_PUMP, radiator_fan::RADIATOR_FAN,
        stirrer::STIRRER,
    },
    ControlCommunicationResources,
};
use core::time::Duration as CoreDuration;
use defmt::warn;
use embassy_futures::select::{select3, Either3};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use embassy_time::{Duration, Instant, Ticker};
use hoshiguma_protocol::cooler::{
    event::{Event, EventKind},
    rpc::{Request, Response},
    types::{Compressor, CoolantPump, RadiatorFan, Stirrer},
    SERIAL_BAUD,
};
use static_cell::StaticCell;
use teeny_rpc::{server::Server, transport::embedded::EioTransport};

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

    let mut config = UartConfig::default();
    config.baudrate = SERIAL_BAUD;

    let uart = BufferedUart::new(
        r.uart, Irqs, r.tx_pin, r.rx_pin, tx_buffer, rx_buffer, config,
    );

    let transport = EioTransport::new(uart);
    let mut server = Server::<_, Request, Response>::new(transport);

    let radiator_fan_tx = RADIATOR_FAN.sender();
    let compressor_tx = COMPRESSOR.sender();
    let stirrer_tx = STIRRER.sender();
    let coolant_pump_tx = COOLANT_PUMP.sender();

    let mut watchdog = CommunicationWatchdog::new(Duration::from_secs(5));
    let mut watchdog_check_tick = Ticker::every(Duration::from_secs(1));

    loop {
        match select3(
            server.wait_for_request(CoreDuration::from_secs(5)),
            NEW_EVENT.receive(),
            watchdog_check_tick.next(),
        )
        .await
        {
            Either3::First(Ok(request)) => {
                watchdog.feed();

                let response = match request {
                    Request::Ping(i) => Some(Response::Ping(i)),
                    Request::GetSystemInformation => {
                        Some(Response::GetSystemInformation(crate::system_information()))
                    }
                    Request::GetState => {
                        // TODO
                    }
                    Request::SetRadiatorFan(setting) => {
                        radiator_fan_tx.send(setting);
                        Some(Response::SetRadiatorFan)
                    }
                    Request::SetCompressor(setting) => {
                        compressor_tx.send(setting);
                        Some(Response::SetCompressor)
                    }
                    Request::SetStirrer(setting) => {
                        stirrer_tx.send(setting);
                        Some(Response::SetStirrer)
                    }
                    Request::SetCoolantPump(setting) => {
                        coolant_pump_tx.send(setting);
                        Some(Response::SetCoolantPump)
                    }
                };

                if let Some(response) = response {
                    if let Err(e) = server.send_response(response).await {
                        warn!("Server failed sending response: {}", e);
                    }
                }
            }
            Either3::First(Err(e)) => {
                warn!("Server failed waiting for request: {}", e);
            }
            Either3::Second(event) => {}
            Either3::Third(_) => {
                if watchdog.check() == CommunicationWatchdogState::Triggered {
                    warn!("Turning off cooling due to communication watchdog");

                    radiator_fan_tx.send(RadiatorFan::Idle);
                    compressor_tx.send(Compressor::Idle);
                    stirrer_tx.send(Stirrer::Idle);
                    coolant_pump_tx.send(CoolantPump::Idle);
                }
            }
        }
    }
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
