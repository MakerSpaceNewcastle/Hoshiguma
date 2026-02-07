use crate::{TELEMETRY_TX, UsbResources};
use defmt::{info, warn};
use embassy_executor::Spawner;
use embassy_rp::{bind_interrupts, peripherals::USB, usb::InterruptHandler};
use embassy_sync::pubsub::WaitResult;
use embassy_usb::{UsbDevice, driver::EndpointError};
use static_cell::StaticCell;

bind_interrupts!(struct Irqs {
    USBCTRL_IRQ => InterruptHandler<USB>;
});

type Driver = embassy_rp::usb::Driver<'static, USB>;
type CdcAcmClass = embassy_usb::class::cdc_acm::CdcAcmClass<'static, Driver>;

#[embassy_executor::task]
pub(super) async fn task(r: UsbResources, spawner: Spawner) {
    let driver = Driver::new(r.usb, Irqs);

    let config = {
        let mut config = embassy_usb::Config::new(0xc0de, 0xcafe); // TODO
        config.manufacturer = Some("Dan Nixon");
        config.product = Some("Hoshiguma Telemetry Module");
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

        if let Err(e) = echo_telemetry(&mut serial_class).await {
            warn!("USB endpoint error: {}", e);
        }

        info!("USB serial disconnected");
    }
}

async fn echo_telemetry(serial: &mut CdcAcmClass) -> Result<(), EndpointError> {
    let mut sub = TELEMETRY_TX.subscriber().unwrap();

    loop {
        match sub.next_message().await {
            WaitResult::Lagged(_) => unreachable!(),
            WaitResult::Message(msg) => {
                serial.write_packet(msg.as_bytes()).await?;
                serial.write_packet(b"\r\n").await?;
            }
        }
    }
}

#[embassy_executor::task]
async fn usb_task(mut usb: UsbDevice<'static, Driver>) -> ! {
    usb.run().await
}
