#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]
#![deny(clippy::large_stack_frames)]

use esp_backtrace as _;
use esp_hal::clock::CpuClock;
use esp_hal::main;
use esp_hal::time::{Duration, Instant};
//use log::{info, warn};
use log::info;
use esp_hal::twai::BaudRate;

extern crate alloc;

// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[allow(
    clippy::large_stack_frames,
    reason = "it's not unusual to allocate larger buffers etc. in main"
)]
#[main]
fn main() -> ! {
    esp_println::logger::init_logger_from_env();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    esp_alloc::heap_allocator!(#[esp_hal::ram(reclaimed)] size: 65536);

    // CAN 2.0 IO stuff
    let _can3_stb = vcu_firm::can::tranceiver_enable(peripherals.GPIO4);
    let (mut can3_rx, mut can3_tx) = vcu_firm::can::can_init(
        // TODO:
        // Verify bus speed value
        peripherals.TWAI0,  // TWAI0
        peripherals.GPIO14, // RX
        peripherals.GPIO13, // TX
        BaudRate::B500K     // BUS Speed
    );

    loop {
        let delay_start = Instant::now();
        info!("main.rs initializing...");
    
        // Can input checker
        match can3_rx.receive() {
            Ok(frame) => info!("CAN Frame: {frame}"),
            Err(nb::Error::Other(e)) => info!("Can Frame Error: {e}"),
            Err(nb::Error::WouldBlock) => {}
        }

        while delay_start.elapsed() < Duration::from_millis(500) {}
    }
}
