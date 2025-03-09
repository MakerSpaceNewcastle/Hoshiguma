use crate::CoolerCommunicationResources;
use embassy_rp::{
    bind_interrupts,
    peripherals::UART1,
    uart::{Async, Config as UartConfig, InterruptHandler, Uart},
};

bind_interrupts!(pub struct Irqs {
    UART1_IRQ  => InterruptHandler<UART1>;
});

pub(crate) type CoolerUart = Uart<'static, UART1, Async>;

impl From<CoolerCommunicationResources> for CoolerUart {
    fn from(r: CoolerCommunicationResources) -> Self {
        let mut config = UartConfig::default();
        config.baudrate = 9600;

        Uart::new(
            r.uart,
            r.tx_pin,
            r.rx_pin,
            Irqs,
            r.tx_dma_ch,
            r.rx_dma_ch,
            config,
        )
    }
}
