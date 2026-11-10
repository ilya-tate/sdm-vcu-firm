#![no_std]
#![no_main]



use defmt::{info, unwrap};
use embassy_executor::Spawner;
use sdm_vcu_firm::{CAN_MODE, WDG_TIMEOUT_US, can, clocks, wdg};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::Timer;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Clock -> CAN timing calculated from can bus clock
    let peripherals = embassy_stm32::init(clocks::config());

    clocks::verify(&peripherals.RCC);

    wdg::check_reset();

    // Initialize CAN peripheral
    let can = match can::init::can_init(
        peripherals.FDCAN1,
        peripherals.PA11,
        peripherals.PA12,
        CAN_MODE,
    ) {
        Ok(can) => can,
        Err(e) => defmt::panic!("CAN Init Error: {}", e),
    };

    // Spawn CAN tasks
    let (can_tx, can_rx, can_props) = can.split();
    spawner.spawn(unwrap!(can::input_listener(
        can_rx,
        can::channels::IN.dyn_sender(),
    )));
    spawner.spawn(unwrap!(can::input_processor(
        can::channels::IN.dyn_receiver()
    )));
    spawner.spawn(unwrap!(can::output_sender(
        can_tx,
        can::channels::OUT.dyn_receiver(),
        can::channels::OUT.dyn_sender(),
    )));

    Timer::after_secs(1).await;
    info!("SDM VCU init done");

    // Reset if loop stalls
    let mut wdg: IndependentWatchdog<'_, embassy_stm32::peripherals::IWDG> =
        IndependentWatchdog::new(peripherals.IWDG, WDG_TIMEOUT_US);
    wdg.unleash();
    
    // Super loop
    loop {
        #[cfg(debug_assertions)]
        can::queue_msg(can::CAN_ID, &[1, 2, 3]); // Test case
        can::log_bus_errs(&can_props);
        wdg.pet();
        Timer::after_secs(1).await;
    }
}
