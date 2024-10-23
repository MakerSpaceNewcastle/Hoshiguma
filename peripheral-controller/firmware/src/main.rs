#![no_std]
#![no_main]

mod status_lamp;

use defmt::{debug, info, unwrap};
use defmt_rtt as _;
use embassy_executor::{Executor, Spawner};
use embassy_rp::{
    gpio::{Input, Level, Output, OutputOpenDrain, Pull},
    multicore::{spawn_core1, Stack},
    watchdog::Watchdog,
};
use embassy_sync::{
    blocking_mutex::raw::CriticalSectionRawMutex, channel::Channel, signal::Signal,
};
use embassy_time::{Duration, Ticker, Timer};
use one_wire_bus::OneWire;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use static_cell::StaticCell;

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let p = unsafe { embassy_rp::Peripherals::steal() };

    // Blink the on-board LED pretty fast
    let mut led = Output::new(p.PIN_25, Level::Low);
    loop {
        led.toggle();
        embassy_time::block_for(Duration::from_millis(50));
    }
}

static mut CORE1_STACK: Stack<4096> = Stack::new();
static EXECUTOR0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR1: StaticCell<Executor> = StaticCell::new();

static STATUS_LAMP: Signal<CriticalSectionRawMutex, status_lamp::StatusLampSetting> = Signal::new();
static CHANNEL: Channel<CriticalSectionRawMutex, Event, 1> = Channel::new();

enum Event {
    InputChanged,
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_rp::init(Default::default());

    let mut watchdog = Watchdog::new(p.WATCHDOG);
    watchdog.start(Duration::from_millis(550));

    // let in0 = Input::new(p.PIN_15, Pull::Down);
    // let in1 = Input::new(p.PIN_14, Pull::Down);
    // let in2 = Input::new(p.PIN_13, Pull::Down);
    let in3_run_detect = Input::new(p.PIN_12, Pull::Down);
    let in4_air_assist_demand = Input::new(p.PIN_11, Pull::Down);
    let in5_fume_extractor_mode = Input::new(p.PIN_10, Pull::Down);
    let in6_chassis_intrusion = Input::new(p.PIN_9, Pull::Down);
    let in7_24v_detect = Input::new(p.PIN_8, Pull::Down);

    let relay3_control_interrupt = Output::new(p.PIN_17, Level::Low);
    let relay4_laser_enable = Output::new(p.PIN_18, Level::Low);
    // let relay5 = Output::new(p.PIN_19, Level::Low);
    let relay6_air_assist_pump = Output::new(p.PIN_20, Level::Low);
    let relay7_fume_extractor = Output::new(p.PIN_21, Level::Low);

    let status_lamp = new_status_lamp!(p);

    let led = Output::new(p.PIN_25, Level::Low);

    let mut onewire_bus = {
        let pin = OutputOpenDrain::new(p.PIN_22, Level::Low);
        OneWire::new(pin).unwrap()
    };

    for device_address in onewire_bus.devices(false, &mut embassy_time::Delay) {
        let device_address = device_address.unwrap();
        info!("Found one wire device at address: {}", device_address.0);
    }

    spawn_core1(
        p.CORE1,
        unsafe { &mut *core::ptr::addr_of_mut!(CORE1_STACK) },
        move || {
            let executor1 = EXECUTOR1.init(Executor::new());
            executor1.run(|spawner| {
                unwrap!(spawner.spawn(watchdog_feed(watchdog, led)));

                unwrap!(spawner.spawn(status_lamp_task(status_lamp)));
            });
        },
    );

    let executor0 = EXECUTOR0.init(Executor::new());
    executor0.run(|spawner| {
        unwrap!(spawner.spawn(read_temperatures(onewire_bus)));
    });
}

#[embassy_executor::task]
async fn watchdog_feed(mut watchdog: Watchdog, mut led: Output<'static>) {
    loop {
        watchdog.feed();
        led.toggle();
        // STATUS_LAMP.signal(StatusLamp::r#static(true, true, false));
        Timer::after_millis(500).await;

        watchdog.feed();
        led.toggle();
        // STATUS_LAMP.signal(StatusLamp::amber());
        Timer::after_millis(500).await;
    }
}

#[derive(Default)]
struct PinChangeDetector {
    last: Option<Level>,
}

impl PinChangeDetector {
    fn update(&mut self, new: Level) -> Option<Level> {
        let changed = self.last != Some(new);
        self.last = Some(new);

        if changed {
            self.last
        } else {
            None
        }
    }
}

#[embassy_executor::task(pool_size = 8)]
async fn watch_input(input: Input<'static>, num: usize) {
    let mut detector = PinChangeDetector::default();

    loop {
        Timer::after_micros(10).await;

        if let Some(level) = detector.update(input.get_level()) {
            // TODO
        }
    }
}

#[embassy_executor::task]
async fn status_lamp_task(mut status_lamp: status_lamp::StatusLamp) {
    loop {
        let settings = STATUS_LAMP.wait().await;
        status_lamp.output(&settings);
    }
}

#[embassy_executor::task]
async fn read_temperatures(mut bus: OneWire<OutputOpenDrain<'static>>) {
    let mut delay = embassy_time::Delay;

    let mut ticker = Ticker::every(Duration::from_secs(5));

    loop {
        ds18b20::start_simultaneous_temp_measurement(&mut bus, &mut delay).unwrap();

        Timer::after_millis(ds18b20::Resolution::Bits12.max_measurement_time_millis() as u64).await;

        let mut search_state = None;
        while let Some((device_address, state)) = bus
            .device_search(search_state.as_ref(), false, &mut delay)
            .unwrap()
        {
            search_state = Some(state);

            if device_address.family_code() == ds18b20::FAMILY_CODE {
                debug!("Found DS18B20 at address: {}", device_address.0);

                let sensor = ds18b20::Ds18b20::new::<()>(device_address).unwrap();
                let sensor_data = sensor.read_data(&mut bus, &mut delay).unwrap();
                info!(
                    "DS18B20 {} is {}°C",
                    device_address.0, sensor_data.temperature
                );
            } else {
                info!(
                    "Found unknown one wire device at address: {}",
                    device_address.0
                );
            }
        }

        ticker.next().await;
    }
}
