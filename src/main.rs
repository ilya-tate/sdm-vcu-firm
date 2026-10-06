#![no_std]
#![no_main]

mod can;

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
// Repo imports
use can::try_frame_build;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let config = Config::default();

    let _pin = embassy_stm32::init(config);

    Timer::after_secs(1).await;
    info!("SDM VCU init done");

    // Test frame
    let _ = try_frame_build(0x123, &[1, 2, 3]);
}
