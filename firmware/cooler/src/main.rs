#![no_std]
#![no_main]

mod devices;
mod machine;
mod network;

use assign_resources::assign_resources;
use defmt::info;
use defmt_rtt as _;
use devices::{
    compressor::Compressor, coolant_flow_sensor::CoolantFlowSensor, coolant_pump::CoolantPump,
    radiator_fan::RadiatorFan, temperature_sensors::TemperatureSensors,
};
use embassy_executor::Spawner;
use embassy_rp::{
    Peri,
    gpio::{Level, Output},
    peripherals::{self},
    watchdog::Watchdog,
};
use embassy_sync::blocking_mutex::raw::{CriticalSectionRawMutex, RawMutex};
use embassy_time::{Duration, Timer};
use hoshiguma_api::BootReason;
use machine::Machine;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use portable_atomic as _;
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_19,
    },
    onewire: OnewireResources {
        pin: PIN_28,
    },
    flow_sensor: FlowSensorResources {
        pwm: PWM_SLICE7,
        pin: PIN_5,
    },
    compressor: CompressorResources {
        relay: PIN_8,
    },
    coolant_pump: CoolantPumpResources {
        relay: PIN_9,
    },
    radiator_fan: RadiatorFanResources {
        relay: PIN_10,
    },
    ethernet: EthernetResources {
        pio: PIO0,
        mosi: PIN_23,
        miso: PIN_22,
        sck: PIN_21,
        tx_dma: DMA_CH0,
        rx_dma: DMA_CH1,
        cs: PIN_20,
        int: PIN_24,
        reset: PIN_25,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let p = unsafe { embassy_rp::Peripherals::steal() };
    let r = split_resources!(p);

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed();

        // Blink the on-board LED pretty fast
        led.toggle();

        embassy_time::block_for(Duration::from_millis(50));
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let p = embassy_rp::init(Default::default());
    let r = split_resources!(p);

    info!("Version: {}", git_version::git_version!());
    info!("Boot reason: {}", boot_reason());

    let machine = Machine {
        coolant_pump: CoolantPump::new(r.coolant_pump),
        compressor: Compressor::new(r.compressor),
        radiator_fan: RadiatorFan::new(r.radiator_fan),
        coolant_flow_sensor: CoolantFlowSensor::new(&spawner, r.flow_sensor),
        temperature_sensors: TemperatureSensors::new(&spawner, r.onewire),
    };

    static CUNT: StaticCell<BiDirectionalChannel<CriticalSectionRawMutex, usize, 8, 4, 4>> =
        StaticCell::new();
    let cunt = CUNT.init(BiDirectionalChannel::new());

    let mut side_a_1 = cunt.side_a();
    let side_b_1 = cunt.side_b();

    spawner.must_spawn(network::task(spawner, r.ethernet));

    spawner.must_spawn(watchdog_feed_task(r.status, side_b_1));

    #[cfg(feature = "test-panic-on-core-0")]
    spawner.must_spawn(dummy_panic());

    loop {
        match side_a_1.inbox.next_message().await {
            embassy_sync::pubsub::WaitResult::Lagged(_) => todo!(),
            embassy_sync::pubsub::WaitResult::Message(i) => info!("fucking i = {}", i),
        }
    }
}

#[embassy_executor::task]
async fn watchdog_feed_task(
    r: StatusResources,
    chan: Side<'static, CriticalSectionRawMutex, usize, 8, 4, 4>,
) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_millis(600));

    let mut i = 0_usize;
    loop {
        watchdog.feed();
        onboard_led.toggle();
        Timer::after_millis(500).await;

        chan.outbox.publish(i).await;
        i = i.wrapping_add(1);
    }
}

#[embassy_executor::task]
async fn dummy_panic() {
    embassy_time::Timer::after_secs(5).await;
    panic!("oh dear, how sad. nevermind...");
}

fn boot_reason() -> BootReason {
    let reason = embassy_rp::pac::WATCHDOG.reason().read();

    if reason.force() {
        BootReason::WatchdogForced
    } else if reason.timer() {
        BootReason::WatchdogTimeout
    } else {
        BootReason::Normal
    }
}

pub struct BiDirectionalChannel<
    M: RawMutex,
    T: Clone,
    const CAP: usize,
    const NUM_A: usize,
    const NUM_B: usize,
> {
    a_to_b: embassy_sync::pubsub::PubSubChannel<M, T, CAP, NUM_B, NUM_A>,
    b_to_a: embassy_sync::pubsub::PubSubChannel<M, T, CAP, NUM_A, NUM_B>,
}

impl<M: RawMutex, T: Clone, const CAP: usize, const NUM_A: usize, const NUM_B: usize>
    BiDirectionalChannel<M, T, CAP, NUM_A, NUM_B>
{
    pub const fn new() -> Self {
        Self {
            a_to_b: embassy_sync::pubsub::PubSubChannel::new(),
            b_to_a: embassy_sync::pubsub::PubSubChannel::new(),
        }
    }

    pub fn side_a<'a>(&'a self) -> Side<'a, M, T, CAP, NUM_A, NUM_B> {
        Side {
            outbox: self.a_to_b.publisher().unwrap(),
            inbox: self.b_to_a.subscriber().unwrap(),
        }
    }

    pub fn side_b<'a>(&'a self) -> Side<'a, M, T, CAP, NUM_B, NUM_A> {
        Side {
            outbox: self.b_to_a.publisher().unwrap(),
            inbox: self.a_to_b.subscriber().unwrap(),
        }
    }
}

pub struct Side<
    'a,
    M: RawMutex,
    T: Clone,
    const CAP: usize,
    const NUM_US: usize,
    const NUM_THEM: usize,
> {
    outbox: embassy_sync::pubsub::Publisher<'a, M, T, CAP, NUM_THEM, NUM_US>,
    inbox: embassy_sync::pubsub::Subscriber<'a, M, T, CAP, NUM_US, NUM_THEM>,
}
