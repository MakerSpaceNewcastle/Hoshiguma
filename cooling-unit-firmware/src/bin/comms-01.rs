#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_rp::{
    peripherals::UART0,
    uart::{BufferedInterruptHandler, BufferedUartRx, BufferedUartTx, Parity},
};
use embassy_sync::{blocking_mutex::raw::ThreadModeRawMutex, mutex::Mutex};
use postcard_rpc::{
    define_dispatch,
    header::VarHeader,
    server::{
        impls::embedded_io_async_v0_6::{
            dispatch_impl::{WireRxBuf, WireRxImpl, WireTxImpl},
            EioWireRx, EioWireTx, EioWireTxInner,
        },
        Dispatch, Server,
    },
};
use static_cell::{ConstStaticCell, StaticCell};
use workbook_icd::{PingEndpoint, ENDPOINT_LIST, TOPICS_IN_LIST, TOPICS_OUT_LIST};
use {defmt_rtt as _, panic_probe as _};

pub struct Context;

type AppTx = WireTxImpl<ThreadModeRawMutex, BufferedUartTx<'static, UART0>>;
type AppRx = WireRxImpl<BufferedUartRx<'static, UART0>>;
type AppServer = Server<AppTx, AppRx, WireRxBuf, MyApp>;

define_dispatch! {
    app: MyApp;
    spawn_fn: spawn_fn;
    tx_impl: AppTx;
    spawn_impl: postcard_rpc::server::impls::embedded_io_async_v0_6::dispatch_impl::WireSpawnImpl;
    context: Context;

    endpoints: {
        list: ENDPOINT_LIST;

        | EndpointTy                | kind      | handler                       |
        | ----------                | ----      | -------                       |
        | PingEndpoint              | blocking  | ping_handler                  |
    };
    topics_in: {
        list: TOPICS_IN_LIST;

        | TopicTy                   | kind      | handler                       |
        | ----------                | ----      | -------                       |
    };
    topics_out: {
        list: TOPICS_OUT_LIST;
    };
}

embassy_rp::bind_interrupts!(struct Irqs {
    UART0_IRQ => BufferedInterruptHandler<UART0>;
});

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    info!("Start");
    let p = embassy_rp::init(Default::default());

    static TX_BUF: ConstStaticCell<[u8; 4096]> = ConstStaticCell::new([0u8; 4096]);
    static RX_BUF: ConstStaticCell<[u8; 4096]> = ConstStaticCell::new([0u8; 4096]);

    let u = {
        let mut config = embassy_rp::uart::Config::default();
        config.parity = Parity::ParityNone;
        config.baudrate = 9600;
        embassy_rp::uart::BufferedUart::new(
            p.UART0,
            Irqs,
            p.PIN_0,
            p.PIN_1,
            TX_BUF.take(),
            RX_BUF.take(),
            config,
        )
    };

    static TX_BUF2: ConstStaticCell<[u8; 4096]> = ConstStaticCell::new([0u8; 4096]);
    static RX_BUF2: ConstStaticCell<[u8; 4096]> = ConstStaticCell::new([0u8; 4096]);
    static RX_BUF3: ConstStaticCell<[u8; 4096]> = ConstStaticCell::new([0u8; 4096]);

    let (etx, erx) = u.split();

    static TX_STO: StaticCell<
        Mutex<ThreadModeRawMutex, EioWireTxInner<BufferedUartTx<'static, UART0>>>,
    > = StaticCell::new();

    let inner = EioWireTxInner {
        t: etx,
        tx_buf: TX_BUF2.take(),
        log_seq: 0,
    };

    let tx_impl = EioWireTx {
        t: &*TX_STO.init(Mutex::new(inner)),
    };

    let rx_impl = EioWireRx {
        remain: RX_BUF2.take(),
        offset: 0,
        rx: erx,
    };

    let context = Context;
    let dispatcher = MyApp::new(context, spawner.into());
    let vkk = dispatcher.min_key_len();
    let mut server: AppServer = Server::new(tx_impl, rx_impl, RX_BUF3.take(), dispatcher, vkk);

    loop {
        let _ = server.run().await;
    }
}

fn ping_handler(_context: &mut Context, _header: VarHeader, rqst: u32) -> u32 {
    info!("ping");
    rqst
}
