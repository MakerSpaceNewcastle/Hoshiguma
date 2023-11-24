mod retry;
mod sensors;
mod services;
mod wifi;

use crate::{
    sensors::{
        coolant_level::CoolantLevelSensor, frequency_counter::FrequencyCounter,
        temperature::TemperatureSensors, SensorReadAndUpdate,
    },
    services::MqttService,
};
use channel_bridge::notification::Notification;
use embassy_time::{Duration, Ticker, Timer};
use esp_idf_hal::{
    delay::Delay,
    gpio::{AnyIOPin, PinDriver},
    prelude::Peripherals,
    task::executor::EspExecutor,
    uart,
    units::Hertz,
};
use esp_idf_svc::eventloop::EspSystemEventLoop;
use esp_idf_sys as _;
use koishi_telemetry_protocol as protocol;
use log::{error, info};
use one_wire_bus::OneWire;
use smart_leds::{SmartLedsWrite, RGB8};
use ws2812_esp32_rmt_driver::Ws2812Esp32Rmt;

pub static QUIT: Notification = Notification::new();

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::timer::embassy_time::queue::link();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    info!("Hello, world!");

    let koishi_telemetry_uart: uart::UartDriver = uart::UartDriver::new(
        peripherals.uart1,
        peripherals.pins.gpio0,
        peripherals.pins.gpio1,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &uart::config::Config::default().baudrate(Hertz(57600)),
    )
    .expect("koishi telemetry UART should be configured");

    let mut led = Ws2812Esp32Rmt::new(0, 8).expect("WS2812 driver should be configured");
    led.write([RGB8::new(8, 0, 0)].into_iter()).unwrap();
    Delay::delay_ms(250);
    led.write([RGB8::new(0, 8, 0)].into_iter()).unwrap();
    Delay::delay_ms(250);
    led.write([RGB8::new(0, 0, 8)].into_iter()).unwrap();
    Delay::delay_ms(250);
    led.write([RGB8::new(8, 8, 8)].into_iter()).unwrap();

    let mut pin =
        PinDriver::output(peripherals.pins.gpio18).expect("koishi enable pin should be configured");
    pin.set_low().unwrap();
    Delay::delay_ms(1000);
    pin.set_high().unwrap();
    Delay::delay_ms(1000);
    pin.set_low().unwrap();
    Delay::delay_ms(1000);
    pin.set_high().unwrap();
    Delay::delay_ms(1000);
    pin.set_low().unwrap();
    Delay::delay_ms(1000);
    pin.set_high().unwrap();
    Delay::delay_ms(1000);

    let wifi = wifi::setup("Maker Space", "donotbeonfire", peripherals.modem, sysloop);

    let mqtt = MqttService::new();

    let mut pin = PinDriver::input(peripherals.pins.gpio4)
        .expect("coolant flow sensor pin should be configured");
    let coolant_flow = FrequencyCounter::new(&mut pin, 1.0);

    let mut pin = PinDriver::input(peripherals.pins.gpio5)
        .expect("coolant pump speed sensor pin should be configured");
    let coolant_pump_speed = FrequencyCounter::new(&mut pin, 1.0);

    let upper = PinDriver::input(peripherals.pins.gpio6)
        .expect("upper coolant level sensor pin should be configured");
    let lower = PinDriver::input(peripherals.pins.gpio7)
        .expect("lower coolant level sensor pin should be configured");
    let coolant_level = CoolantLevelSensor::new(upper, lower);

    let pin = PinDriver::input_output_od(peripherals.pins.gpio19)
        .expect("one wire bus pin should be configured");
    let mut temperature_sensors = TemperatureSensors::new(
        OneWire::new(pin).expect("one wire bus dirver should be created"),
        Delay,
    );
    temperature_sensors.debug_scan();

    let executor = EspExecutor::<16, _>::new();
    let mut tasks = heapless::Vec::<_, 16>::new();
    executor
        .spawn_local_collect(task_coolant_flow(coolant_flow), &mut tasks)
        .expect("coolant flow sensor task should be spawned")
        .spawn_local_collect(task_coolant_pump_speed(coolant_pump_speed), &mut tasks)
        .expect("coolant pump speed sensor task should be spawned")
        .spawn_local_collect(
            task_coolant_level_sensor(coolant_level, mqtt.clone()),
            &mut tasks,
        )
        .expect("coolant level sensor task should be spawned")
        .spawn_local_collect(
            task_temperature_sensors(temperature_sensors, mqtt),
            &mut tasks,
        )
        .expect("temperature sensor task should be spawned")
        .spawn_local_collect(demo_koishi_telemetry(koishi_telemetry_uart), &mut tasks)
        .expect("koishi telemetry task should be spawned")
        .spawn_local_collect(wifi::task(wifi), &mut tasks)
        .expect("wifi task should be spawned");
    executor.run_tasks(move || !QUIT.triggered(), tasks);

    Ok(())
}

async fn task_coolant_flow(counter: FrequencyCounter) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut then = embassy_time::Instant::now();

    loop {
        let now = embassy_time::Instant::now();
        let duration = now - then;

        let counts = counter.measure();
        info!("flow: {counts} counts in {duration}");

        then = now;

        ticker.next().await;
    }
}

async fn task_coolant_pump_speed(counter: FrequencyCounter) {
    let mut ticker = Ticker::every(Duration::from_secs(1));
    let mut then = embassy_time::Instant::now();

    loop {
        let now = embassy_time::Instant::now();
        let duration = now - then;

        let counts = counter.measure();
        info!("rpm: {counts} counts in {duration}");

        then = now;

        ticker.next().await;
    }
}

async fn task_coolant_level_sensor(mut sensors: impl SensorReadAndUpdate, _mqtt: MqttService) {
    let mut ticker = Ticker::every(Duration::from_secs(5));

    loop {
        sensors.read();

        ticker.next().await;
    }
}

async fn task_temperature_sensors(mut sensors: impl SensorReadAndUpdate, mqtt: MqttService) {
    let mut ticker = Ticker::every(Duration::from_secs(5));

    loop {
        sensors.read();

        mqtt.test();

        ticker.next().await;
    }
}

async fn demo_koishi_telemetry(uart: uart::UartDriver<'_>) {
    let mut rx_buffer = Vec::<u8>::new();

    loop {
        let mut buf = [0u8];
        match uart.read(&mut buf, 50) {
            Ok(0) => {
                Timer::after(Duration::from_millis(250)).await;
            }
            Ok(_) => {
                let c = buf[0];
                rx_buffer.push(c);

                if c == 0 {
                    info!("end of packet: {:?}", rx_buffer);
                    match postcard::from_bytes_cobs::<protocol::Message>(&mut rx_buffer) {
                        Ok(msg) => {
                            info!("msg: {:#?}", msg);
                        }
                        Err(err) => {
                            error!("Somethings fucky... {}", err);
                        }
                    }
                    rx_buffer.clear();
                }
            }
            Err(err) => {
                error!("Somethings fucky... {}", err);
            }
        }
    }
}
