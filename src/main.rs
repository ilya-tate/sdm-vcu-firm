#![no_std]
#![no_main]

mod can;
mod clock;
mod wdg;

use defmt::{info, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::can::OperatingMode;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};
// Repo imports
use can::msg_handler::CAN_ID;

// TIMEOUT MUST OUTLAST EXPECTED LOOP ITERATION TIME
const WDG_TIMEOUT_US: u32 = 2_000_000;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Clock -> CAN timing calculated from can bus clock
    let peripherals = embassy_stm32::init(clock::config());

    wdg::check_reset();

    let Some(can) = can::try_init(
        peripherals.FDCAN1,
        peripherals.PA11,
        peripherals.PA12,
        OperatingMode::InternalLoopbackMode,
    ) else {
        defmt::panic!("CAN INIT FAILURE");
    };

    let (mut can_tx, can_rx, _) = can.split();
    spawner.spawn(unwrap!(can::input_listener(can_rx)));

    Timer::after_secs(1).await;
    info!("SDM VCU init done");

    // Reset if loop stalls
    let mut wdg = IndependentWatchdog::new(peripherals.IWDG, WDG_TIMEOUT_US);
    wdg.unleash();

    // Super loop
    loop {
        can::try_msg(&mut can_tx, CAN_ID, &[1, 2, 3]).await;
        wdg.pet();
        Timer::after_secs(1).await;
    }
}
