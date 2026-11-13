#![no_std]
#![no_main]

mod can;
mod clocks;
mod wdg;

use crate::can::devices::motor_controller::MotorController;

use defmt::{println, trace, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::can::OperatingMode;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::Timer;

use {defmt_rtt as _, panic_probe as _};

// TIMEOUT MUST OUTLAST EXPECTED LOOP ITERATION TIME
const WDG_TIMEOUT_US: u32 = 2_000_000;

#[cfg(debug_assertions)]
const CAN_MODE: OperatingMode = OperatingMode::InternalLoopbackMode;
#[cfg(not(debug_assertions))]
const CAN_MODE: OperatingMode = OperatingMode::NormalOperationMode;

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    // Clock -> CAN timing calculated from can bus clock
    let peripherals = embassy_stm32::init(clocks::config());

    clocks::verify(&peripherals.RCC);

    wdg::check_reset();

    let can = match can::init::can_init(
        peripherals.FDCAN1,
        peripherals.PA11,
        peripherals.PA12,
        CAN_MODE,
    ) {
        Ok(can) => can,
        Err(e) => defmt::panic!("CAN Init Error: {}", e),
    };

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

    // Reset if loop stalls
    let mut wdg: IndependentWatchdog<'_, embassy_stm32::peripherals::IWDG> =
        IndependentWatchdog::new(peripherals.IWDG, WDG_TIMEOUT_US);
    wdg.unleash();

    // Test frames
    // Sent when "--release" is omitted from cargo run
    #[cfg(debug_assertions)]
    spawner.spawn(unwrap!(can::test_frames::send_test_frames()));

    MotorController.clear_faults();
    MotorController.read_config();
    MotorController.send_stop_cmd();

    println!("SDM VCU init done");

    let mut tick: u32 = 0;
    let mut log: u32 = 0;

    // Super loop
    loop {
        wdg.pet();
        Timer::after_secs(1).await;

        // Error logger matches controller send rate
        if tick.is_multiple_of(2) {
            trace!("Log: {}", log);
            log += 1;
            can::log_bus_errs(&can_props);
            MotorController.clear_faults();
        }
        tick = tick.wrapping_add(1);
    }
}
