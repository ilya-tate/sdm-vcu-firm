#![no_std]
#![no_main]

mod can;
mod clocks;
mod init;
mod pedals;

use crate::can::devices::motor_controller::MotorController;

use defmt::{error, println, trace};
use embassy_executor::Spawner;
use embassy_time::{Duration, Instant};

use {defmt_rtt as _, panic_probe as _};

const START_DELAY: Duration = Duration::from_secs(5);

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    error!("BENCH BUILD: PEDALS IGNORED, DO NOT DRIVE");

    let init::Resources {
        mut wdg,
        can_props,
        mut ticker,
    } = init::setup(spawner);
    let mut tick: u32 = 0;
    let mut log: u32 = 0;
    let mut ready_at: Option<Instant> = None;

    // Super loop
    loop {
        wdg.pet();
        ticker.next().await;

        // Grab the globals each cycle
        let torque = 100;
        // Constantly read and send torque to motor
        // TODO: Use w/ state machine
        if MotorController.is_ready() {
            if ready_at.get_or_insert_with(Instant::now).elapsed() < START_DELAY {
                MotorController.send_stop_cmd();
            } else if MotorController.is_enabled() {
                MotorController.send_torque_cmd(torque);
            } else {
                MotorController.send_start_cmd();
            }
        } else {
            ready_at = None;
            MotorController.send_stop_cmd();
        }

        if tick.is_multiple_of(100) {
            println!(
                "Torque: {}\nReady: {}\nEnabled: {}",
                torque,
                MotorController.is_ready(),
                MotorController.is_enabled()
            );
        }

        // Error logger matches controller send rate
        if tick.is_multiple_of(200) {
            trace!("Log: {}", log);
            log += 1;
            can::log_bus_errs(&can_props);
            if torque == 0 {
                MotorController.clear_faults();
            }
        }
        tick = tick.wrapping_add(1);
    }
}
