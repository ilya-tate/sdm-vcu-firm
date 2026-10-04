#![no_std]
#![no_main]

use defmt::info;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

// mod libs

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let mut config = Config::default();

    let pin = embassy_stm32::init(config);

    Timer::after_secs(1).await;
    info!("SDM VCU init done");
}
