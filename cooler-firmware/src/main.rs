#![no_std]
#![no_main]

use assign_resources::assign_resources;
use serde::{Deserialize, Serialize};
use core::sync::atomic::Ordering;
use defmt::{debug, info, unwrap, warn};
use defmt_rtt as _;
use embassy_executor::raw::Executor;
use embassy_rp::{
    bind_interrupts, gpio::{Input, Level, Output, Pull}, uart::BufferedUart, watchdog::Watchdog
};
use embassy_time::{Delay, Duration, Instant, Ticker, Timer};
use git_version::git_version;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::peripherals::{self, PicoPlc};
use portable_atomic::AtomicU64;
use static_cell::StaticCell;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    onewire: OnewireResources {
        pin: ONEWIRE,
    },
    flow_sensor: FlowSensorResources {
        pwm: PWM_SLICE7,
        pin: IN_0,
    },
    resevoir_level: ResevoirLevelSensorResources {
        low: IN_1,
    },
    header_tank_level: HeaderTankLevelSensorResources {
        empty: IN_2,
        low : IN_3,
    },
    relays: RelayOutputResources {
        compressor: RELAY_0,
        stirrer: RELAY_1,
        fan: RELAY_2,
        pump: RELAY_3,
    },
    communication: ControlCommunicationResources {
        tx_pin: IO_0,
        rx_pin: IO_1,
        uart: UART0,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    let p = unsafe { PicoPlc::steal() };
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

static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
static SLEEP_TICKS_CORE_0: AtomicU64 = AtomicU64::new(0);

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = PicoPlc::default();
    let r = split_resources!(p);

    info!("Version: {}", git_version!());

    // Unused IO
    let _in5 = Input::new(p.IN_5, Pull::Down);
    let _in6 = Input::new(p.IN_6, Pull::Down);
    let _in7 = Input::new(p.IN_7, Pull::Down);
    let _relay4 = Output::new(p.RELAY_4, Level::Low);
    let _relay5 = Output::new(p.RELAY_5, Level::Low);
    let _relay6 = Output::new(p.RELAY_6, Level::Low);
    let _relay7 = Output::new(p.RELAY_7, Level::Low);

    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    let spawner = executor_0.spawner();

    unwrap!(spawner.spawn(watchdog_feed_task(r.status)));

    // TODO
    unwrap!(spawner.spawn(rpc_server_task(r.communication)));
    unwrap!(spawner.spawn(read_temperature_sensors(r.onewire)));
    // unwrap!(spawner.spawn(get_fucking_cold(r.relays)));
    // unwrap!(spawner.spawn(fuck_about_with_relays(r.relays)));
    unwrap!(spawner.spawn(measure_dat_pwm(r.flow_sensor)));

    // CPU usage reporting
    unwrap!(spawner.spawn(report_cpu_usage()));

    #[cfg(feature = "test-panic-on-core-0")]
    unwrap!(spawner.spawn(dummy_panic()));

    loop {
        let before = Instant::now().as_ticks();
        cortex_m::asm::wfe();
        let after = Instant::now().as_ticks();
        SLEEP_TICKS_CORE_0.fetch_add(after - before, Ordering::Relaxed);
        unsafe { executor_0.poll() };
    }
}

#[embassy_executor::task]
async fn watchdog_feed_task(r: StatusResources) {
    let mut onboard_led = Output::new(r.led, Level::Low);

    let mut watchdog = Watchdog::new(r.watchdog);
    watchdog.start(Duration::from_millis(600));

    loop {
        watchdog.feed();
        onboard_led.toggle();
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn report_cpu_usage() {
    let mut previous_tick = 0u64;
    let mut previous_sleep_tick_core_0 = 0u64;

    let mut ticker = Ticker::every(Duration::from_secs(1));

    loop {
        ticker.next().await;

        let current_tick = Instant::now().as_ticks();
        let tick_difference = (current_tick - previous_tick) as f32;

        let current_sleep_tick_core_0 = SLEEP_TICKS_CORE_0.load(Ordering::Relaxed);

        let calc_cpu_usage = |current_sleep_tick: u64, previous_sleep_tick: u64| -> f32 {
            let sleep_tick_difference = (current_sleep_tick - previous_sleep_tick) as f32;
            1f32 - sleep_tick_difference / tick_difference
        };

        let usage_core_0 = calc_cpu_usage(current_sleep_tick_core_0, previous_sleep_tick_core_0);

        previous_tick = current_tick;
        previous_sleep_tick_core_0 = current_sleep_tick_core_0;

        info!("Usage: core 0 = {}", usage_core_0);
    }
}

#[embassy_executor::task]
async fn dummy_panic() {
    embassy_time::Timer::after_secs(5).await;
    panic!("oh dear, how sad. nevermind...");
}

#[embassy_executor::task]
async fn measure_dat_pwm(r: FlowSensorResources) {
    let cfg: embassy_rp::pwm::Config = Default::default();
    let pwm = embassy_rp::pwm::Pwm::new_input(
        r.pwm,
        r.pin,
        Pull::Down,
        embassy_rp::pwm::InputMode::RisingEdge,
        cfg,
    );

    let mut ticker = Ticker::every(Duration::from_millis(500));

    loop {
        info!("Input frequency: {} Hz", pwm.counter());
        pwm.set_counter(0);
        ticker.next().await;
    }
}

#[embassy_executor::task]
async fn get_fucking_cold(r: RelayOutputResources) {
    let mut fan = Output::new(r.fan, Level::Low);
    let mut compressor = Output::new(r.compressor, Level::Low);
    let mut stirrer = Output::new(r.stirrer, Level::Low);
    let _pump = Output::new(r.pump, Level::Low);

    fan.set_high();
    Timer::after_secs(2).await;
    stirrer.set_high();
    Timer::after_secs(2).await;
    compressor.set_high();

    loop {
        Timer::after_secs(60).await;
    }
}

#[embassy_executor::task]
async fn fuck_about_with_relays(r: RelayOutputResources) {
    let mut fan = Output::new(r.fan, Level::Low);
    let mut compressor = Output::new(r.compressor, Level::Low);
    let mut stirrer = Output::new(r.stirrer, Level::Low);
    let mut pump = Output::new(r.pump, Level::Low);

    let mut ticker = Ticker::every(Duration::from_secs(5));

    loop {
        fan.toggle();
        ticker.next().await;
        fan.toggle();

        compressor.toggle();
        ticker.next().await;
        compressor.toggle();

        stirrer.toggle();
        ticker.next().await;
        stirrer.toggle();

        pump.toggle();
        ticker.next().await;
        pump.toggle();
    }
}

#[embassy_executor::task]
async fn read_temperature_sensors(r: OnewireResources) {
    let mut bus = pico_plc_bsp::onewire::new(r.pin).unwrap();

    for device_address in bus.devices(false, &mut Delay) {
        let device_address = device_address.unwrap();
        info!("Found one wire device at address: {}", device_address.0);
    }

    let mut ticker = Ticker::every(Duration::from_secs(5));

    loop {
        ds18b20::start_simultaneous_temp_measurement(&mut bus, &mut Delay).unwrap();

        Timer::after_millis(ds18b20::Resolution::Bits12.max_measurement_time_millis() as u64).await;

        let mut search_state = None;
        while let Some((device_address, state)) = bus
            .device_search(search_state.as_ref(), false, &mut Delay)
            .unwrap()
        {
            search_state = Some(state);

            if device_address.family_code() == ds18b20::FAMILY_CODE {
                debug!("Found DS18B20 at address: {}", device_address.0);

                let sensor = ds18b20::Ds18b20::new::<()>(device_address).unwrap();
                let sensor_data = sensor.read_data(&mut bus, &mut Delay).unwrap();
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

bind_interrupts!(struct Irqs {
    UART0_IRQ  => embassy_rp::uart::BufferedInterruptHandler<peripherals::UART0>;
});

#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum Request {
    Ping(u32),
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
enum Response {
    Ping(u32),
}

#[embassy_executor::task]
async fn rpc_server_task(r: ControlCommunicationResources) {
    static TX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let tx_buf = &mut TX_BUF.init([0; 16])[..];
    static RX_BUF: StaticCell<[u8; 16]> = StaticCell::new();
    let rx_buf = &mut RX_BUF.init([0; 16])[..];

    let mut config = embassy_rp::uart::Config::default();
    config.baudrate = 115_200;

    let uart = BufferedUart::new(
        r.uart,
        Irqs,
        r.tx_pin,
        r.rx_pin,
        tx_buf,
        rx_buf,
        config,
    );

    let transport = teeny_rpc::transport::embedded::EioTransport::new(uart);
    let mut server = teeny_rpc::server::Server::<_,Request,Response>::new(transport);

    loop {
        match server
            .wait_for_request(core::time::Duration::from_secs(5))
            .await
        {
            Ok(request) => {
                let response = match request {
                    Request::Ping(i) => Response::Ping(i),
                };
                if let Err(e) = server.send_response(response).await {
                    warn!("Server failed sending response: {}", e);
                }
            }
            Err(e) => {
                warn!("Server failed waiting for request: {}", e);
            }
        }
    }
}
