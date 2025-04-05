use crate::CoolerCommunicationResources;
use defmt::{info, unwrap, warn, Format};
use embassy_futures::select::{select, Either};
use embassy_rp::{
    bind_interrupts,
    peripherals::UART1,
    uart::{BufferedInterruptHandler, BufferedUart, Config as UartConfig},
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex,
    pubsub::{PubSubChannel, WaitResult},
    watch::Watch,
};
use embassy_time::{Duration, Timer};
use hoshiguma_protocol::cooler::{
    event::{EventKind, ObservationEvent},
    rpc::{Request, Response},
    types::{
        Compressor, CoolantPump, HeaderTankCoolantLevelReading, HeatExchangeFluidLevel,
        RadiatorFan, Stirrer,
    },
};
use static_cell::StaticCell;
use teeny_rpc::{client::Client, transport::embedded::EioTransport};

#[derive(Debug, Clone, Format)]
pub(crate) enum CoolerControlCommand {
    SetRadiatorFan(RadiatorFan),
    SetCompressor(Compressor),
    SetStirrer(Stirrer),
    SetCoolantPump(CoolantPump),
}

impl From<CoolerControlCommand> for Request {
    fn from(cmd: CoolerControlCommand) -> Self {
        match cmd {
            CoolerControlCommand::SetRadiatorFan(radiator_fan) => {
                Self::SetRadiatorFan(radiator_fan)
            }
            CoolerControlCommand::SetCompressor(compressor) => Self::SetCompressor(compressor),
            CoolerControlCommand::SetStirrer(stirrer) => Self::SetStirrer(stirrer),
            CoolerControlCommand::SetCoolantPump(coolant_pump) => {
                Self::SetCoolantPump(coolant_pump)
            }
        }
    }
}

pub(crate) static COOLER_CONTROL: PubSubChannel<
    CriticalSectionRawMutex,
    CoolerControlCommand,
    8,
    1,
    1,
> = PubSubChannel::new();

pub(crate) static HEADER_TANK_COOLANT_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    HeaderTankCoolantLevelReading,
    1,
> = Watch::new();

pub(crate) static HEAT_EXCHANGER_FLUID_LEVEL_CHANGED: Watch<
    CriticalSectionRawMutex,
    HeatExchangeFluidLevel,
    1,
> = Watch::new();

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

    let mut control_rx = unwrap!(COOLER_CONTROL.subscriber());

    const SHORT_EVENT_POLL: Duration = Duration::from_millis(50);
    const LONG_EVENT_POLL: Duration = Duration::from_millis(500);

    let mut event_poll_interval = LONG_EVENT_POLL;

    let tx = HEADER_TANK_COOLANT_LEVEL_CHANGED.sender();
    let tx2 = HEAT_EXCHANGER_FLUID_LEVEL_CHANGED.sender();

    loop {
        match select(Timer::after(event_poll_interval), control_rx.next_message()).await {
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
                        match event.kind {
                            EventKind::Boot(_) => {}
                            EventKind::Observation(event) => {
                                match event {
                                    ObservationEvent::Temperatures(v) => {}
                                    ObservationEvent::CoolantFlow(v) => {}
                                    ObservationEvent::HeatExchangeFluidLevel(v) => {
                                        tx2.send(v);
                                    }
                                    ObservationEvent::HeaderTankCoolantLevel(v) => {
                                        tx.send(v);
                                    }
                                }
                                todo!()
                            }
                            EventKind::Control(_) => {}
                        }

                        event_poll_interval = SHORT_EVENT_POLL;
                    }
                    Ok(Response::GetOldestEvent(None)) => {
                        event_poll_interval = LONG_EVENT_POLL;
                    }
                    Ok(_) => {
                        warn!("Unexpected RPC response");
                    }
                    Err(e) => {
                        warn!("RPC error: {}", e);
                    }
                }
            }
            Either::Second(WaitResult::Message(cmd)) => {
                let request: Request = cmd.into();

                // TODO: error handling
                'cmd_send: loop {
                    match client
                        .call(request.clone(), core::time::Duration::from_millis(50))
                        .await
                    {
                        Ok(_) => {
                            break 'cmd_send;
                        }
                        Err(_) => {
                            Timer::after_millis(50).await;
                        }
                    }
                }
            }
            Either::Second(WaitResult::Lagged(_)) => {
                // TODO
            }
        }
    }
}
