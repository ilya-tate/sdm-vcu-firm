#![no_std]
#![no_main]

mod can;
mod clock;

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_stm32::can::OperatingMode;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
// Repo imports
use can::{try_can_init, try_frame_build};

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // Clock -> CAN timing calculated from can bus clock
    let peripherals = embassy_stm32::init(clock::config());

    let Some(mut can) = try_can_init(
        peripherals.FDCAN1,
        peripherals.PA11,
        peripherals.PA12,
        OperatingMode::InternalLoopbackMode,
    ) else {
        return;
    };

    Timer::after_secs(1).await;
    info!("SDM VCU init done");

    // Super loop
    loop {
        if let Some(frame) = try_frame_build(0x123, &[1, 2, 3]) {
            can.write(&frame).await;

            // Waits for frame input
            match can.read().await {
                Ok(envelope) => info!("CAN Rx: {}", envelope.frame),

                Err(e) => error!("CAN Rx ERROR: {}", e),
            }
        }

        Timer::after_secs(1).await;
    }
}
