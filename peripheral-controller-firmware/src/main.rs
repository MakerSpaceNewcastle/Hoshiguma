#![no_std]
#![no_main]

mod changed;
mod cooler;
mod devices;
mod logic;
mod maybe_timer;
mod polled_input;
mod telemetry;

use assign_resources::assign_resources;
use cooler::CoolerUart;
use core::sync::atomic::Ordering;
use defmt::{error, info, unwrap};
use defmt_rtt as _;
use embassy_executor::raw::Executor;
use embassy_rp::{
    gpio::{Input, Level, Output, Pull},
    multicore::{spawn_core1, Stack},
    watchdog::Watchdog,
};
use embassy_time::{Duration, Instant, Ticker, Timer, WithTimeout};
use git_version::git_version;
#[cfg(feature = "panic-probe")]
use panic_probe as _;
use pico_plc_bsp::peripherals::{self, PicoPlc};
use portable_atomic::{AtomicBool, AtomicU64};
use static_cell::StaticCell;
use telemetry::TelemetryUart;

assign_resources! {
    status: StatusResources {
        watchdog: WATCHDOG,
        led: PIN_25,
    },
    onewire: OnewireResources {
        pin: ONEWIRE,
    },
    status_lamp: StatusLampResources {
        red: RELAY_0,
        amber: RELAY_1,
        green: RELAY_2,
    },
    machine_power_detect: MachinePowerDetectResources {
        detect: IN_7,
    },
    chassis_intrusion_detect: ChassisIntrusionDetectResources {
        detect: IN_6,
    },
    air_assist_demand_detect: AirAssistDemandDetectResources {
        detect: IN_4,
    },
    machine_run_detect: MachineRunDetectResources {
        detect: IN_3,
    },
    fume_extraction_mode_switch: FumeExtractionModeSwitchResources {
        switch: IN_5,
    },
    coolant_resevoir_level_sensor: CoolantResevoirLevelSensorResources {
        empty: IN_1,
        low: IN_2,
    },
    air_assist_pump: AirAssistPumpResources {
        relay: RELAY_6,
    },
    fume_extraction_fan: FumeExtractionFanResources {
        relay: RELAY_7,
    },
    laser_enable: LaserEnableResources {
        relay: RELAY_4,
    },
    machine_enable: MachineEnableResources {
        relay: RELAY_3,
    },
    cooler_communication: CoolerCommunicationResources {
        uart: UART1,
        tx_pin: IO_4,
        rx_pin: IO_5,
        tx_dma_ch: DMA_CH2,
        rx_dma_ch: DMA_CH3,
    },
    telemetry: TelemetryResources {
        tx_pin: IO_0,
        rx_pin: IO_1,
        uart: UART0,
        dma_ch: DMA_CH0,
    },
}

#[cfg(not(feature = "panic-probe"))]
#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    use crate::devices::{
        laser_enable::LaserEnableOutput, machine_enable::MachineEnableOutput,
        status_lamp::StatusLampOutput,
    };

    // Flag the panic, indicating that executors should stop scheduling work
    PANIC_HALT.store(true, Ordering::Relaxed);

    let p = unsafe { PicoPlc::steal() };
    let r = split_resources!(p);

    // Disable the machine and laser
    let mut laser_enable = LaserEnableOutput::new(r.laser_enable);
    laser_enable.set_panic();
    let mut machine_enable = MachineEnableOutput::new(r.machine_enable);
    machine_enable.set_panic();

    // Set the status lamp to something distinctive
    let mut status_lamp = StatusLampOutput::new(r.status_lamp);
    status_lamp.set_panic();

    // Report the panic
    let mut uart: TelemetryUart = r.telemetry.into();
    crate::telemetry::report_panic(&mut uart, info);

    let mut watchdog = Watchdog::new(r.status.watchdog);
    let mut led = Output::new(r.status.led, Level::Low);

    loop {
        // Keep feeding the watchdog so that we do not quickly reset.
        // Panics should be properly investigated.
        watchdog.feed();

        // Keep setting the enable and status lamp outputs.
        // Not strictly needed, as no other tasks should be using the outputs at this point, but
        // here for belt and braces.
        laser_enable.set_panic();
        machine_enable.set_panic();
        status_lamp.set_panic();

        // Blink the on-board LED pretty fast
        led.toggle();

        embassy_time::block_for(Duration::from_millis(50));
    }
}

static mut CORE_1_STACK: Stack<4096> = Stack::new();

static EXECUTOR_0: StaticCell<Executor> = StaticCell::new();
static EXECUTOR_1: StaticCell<Executor> = StaticCell::new();

static SLEEP_TICKS_CORE_0: AtomicU64 = AtomicU64::new(0);
static SLEEP_TICKS_CORE_1: AtomicU64 = AtomicU64::new(0);

static PANIC_HALT: AtomicBool = AtomicBool::new(false);

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = PicoPlc::default();
    let r = split_resources!(p);

    info!("Version: {}", git_version!());

    let executor_0 = EXECUTOR_0.init(Executor::new(usize::MAX as *mut ()));
    let spawner = executor_0.spawner();

    // WIP
    unwrap!(spawner.spawn(telem_uart_test(r.telemetry.into())));
    unwrap!(spawner.spawn(cooler_uart_test(r.cooler_communication.into())));

    loop {
        let before = Instant::now().as_ticks();
        cortex_m::asm::wfe();
        let after = Instant::now().as_ticks();
        SLEEP_TICKS_CORE_0.fetch_add(after - before, Ordering::Relaxed);
        if !PANIC_HALT.load(Ordering::Relaxed) {
            unsafe { executor_0.poll() };
        }
    }
}

// WIP
#[embassy_executor::task]
async fn telem_uart_test(mut uart: TelemetryUart) {
    loop {
        let mut b = [0u8];
        if uart.read(&mut b).await.is_ok() {
            info!("telem uart got byte: {}", b);
            uart.write(&b).await.unwrap();
        }
    }
}

// WIP
#[embassy_executor::task]
async fn cooler_uart_test(mut uart: CoolerUart) {
    loop {
        let mut b = [0u8];
        if uart.read(&mut b).await.is_ok() {
            info!("cooler uart got byte: {}", b);
            uart.write(&b).await.unwrap();
        }
    }
}
