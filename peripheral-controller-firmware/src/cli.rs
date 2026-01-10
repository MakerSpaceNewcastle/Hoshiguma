use crate::UsbResources;
use defmt::{debug, info, warn, Format};
use embassy_executor::Spawner;
use embassy_usb::{driver::EndpointError, UsbDevice};
use heapless::Vec;
use pico_plc_bsp::embassy_rp::{bind_interrupts, peripherals::USB, usb::InterruptHandler};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type Driver = pico_plc_bsp::embassy_rp::usb::Driver<'static, USB>;
type CdcAcmClass = embassy_usb::class::cdc_acm::CdcAcmClass<'static, Driver>;

#[embassy_executor::task]
pub(super) async fn task(r: UsbResources, spawner: Spawner) {
    let driver = Driver::new(r.usb, Irqs);

    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe); // TODO
        config.manufacturer = Some("Dan Nixon");
        config.product = Some("Hoshiguma Peripheral Controller");
        config.serial_number = Some("1");
        config.max_power = 100;
        config.max_packet_size_0 = 64;
        config
    };

    let mut builder = {
        static CONFIG_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static BOS_DESCRIPTOR: StaticCell<[u8; 256]> = StaticCell::new();
        static CONTROL_BUF: StaticCell<[u8; 128]> = StaticCell::new();

        embassy_usb::Builder::new(
            driver,
            config,
            CONFIG_DESCRIPTOR.init([0; 256]),
            BOS_DESCRIPTOR.init([0; 256]),
            &mut [], // no msos descriptors
            CONTROL_BUF.init([0; 128]),
        )
    };

    let mut serial_class = {
        static STATE: StaticCell<embassy_usb::class::cdc_acm::State> = StaticCell::new();
        let state = STATE.init(embassy_usb::class::cdc_acm::State::new());
        CdcAcmClass::new(&mut builder, state, 64)
    };

    let usb = builder.build();

    spawner.must_spawn(usb_task(usb));

    loop {
        serial_class.wait_connection().await;
        info!("USB serial connected");

        if let Err(e) = connection_loop(&mut serial_class).await {
            warn!("USB endpoint error in CLI handling: {}", e);
        }

        info!("USB serial disconnected");
    }
}

async fn connection_loop(serial: &mut CdcAcmClass) -> Result<(), EndpointError> {
    let mut line_buf = Vec::<u8, 128>::new();
    let mut rx_buf = [0; 64];

    loop {
        let len = serial.read_packet(&mut rx_buf).await?;
        let rx_data = &rx_buf[..len];
        debug!("Received USB serial data: {}", rx_data);

        for b in rx_data {
            if *b == b'\r' {
                debug!("cmd line data = {}", line_buf.as_slice());
                serial.write_packet(b"\r\n").await?;

                let line: LineParts = line_buf.split(|b| *b == b' ').collect();
                debug!("cmd line = {}", line);
                debug!("num parts: {}", line.len());

                match process_line(line, serial).await {
                    Ok(_) => {}
                    Err(CliFailReason::MalformedInput) => {
                        serial.write_packet(b"RTFM plz.\r\n").await?;
                    }
                    Err(CliFailReason::ExecutionFailure) => {
                        serial
                            .write_packet(b"Command failed. Oh dear. How Sad. Nevermind.\r\n")
                            .await?;
                    }
                    Err(CliFailReason::EndpointError(e)) => {
                        return Err(e);
                    }
                }
                serial.write_packet(b"\r\n").await?;

                line_buf.clear();
            } else {
                line_buf.push(*b).unwrap();
                serial.write_packet(&[*b]).await?;
            }
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver>) -> ! {
    usb.run().await
}

type LineParts<'a> = Vec<&'a [u8], 8>;

#[derive(Debug, Format, PartialEq, Eq)]
enum CliFailReason {
    MalformedInput,
    ExecutionFailure,
    EndpointError(EndpointError),
}

impl From<EndpointError> for CliFailReason {
    fn from(e: EndpointError) -> Self {
        Self::EndpointError(e)
    }
}

async fn process_line<'a>(
    line: LineParts<'a>,
    serial: &mut CdcAcmClass,
) -> Result<(), CliFailReason> {
    use crate::devices::{
        accessories::cooler::{CoolerControlCommand, COOLER_CONTROL_COMMAND},
        air_assist_pump::AIR_ASSIST_PUMP,
    };
    use hoshiguma_protocol::{
        accessories::cooler::types::CoolantPumpState, peripheral_controller::types::AirAssistPump,
    };

    match line[0] {
        b"version" => {
            let version = git_version::git_version!();
            serial.write_packet(version.as_bytes()).await?;
            serial.write_packet(b"\r\n").await?;
            Ok(())
        }
        b"coolant-pump-on" => {
            if let Ok(publisher) = COOLER_CONTROL_COMMAND.publisher() {
                publisher
                    .publish(CoolerControlCommand::CoolantPump(CoolantPumpState::Run))
                    .await;
                Ok(())
            } else {
                Err(CliFailReason::ExecutionFailure)
            }
        }
        b"coolant-pump-off" => {
            if let Ok(publisher) = COOLER_CONTROL_COMMAND.publisher() {
                publisher
                    .publish(CoolerControlCommand::CoolantPump(CoolantPumpState::Idle))
                    .await;
                Ok(())
            } else {
                Err(CliFailReason::ExecutionFailure)
            }
        }
        b"air-pump-on" => {
            AIR_ASSIST_PUMP.sender().send(AirAssistPump::Run);
            Ok(())
        }
        b"air-pump-off" => {
            AIR_ASSIST_PUMP.sender().send(AirAssistPump::Idle);
            Ok(())
        }
        _ => Err(CliFailReason::MalformedInput),
    }
}
