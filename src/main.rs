#![no_std]
#![no_main]

mod can;
mod clocks;
mod init;
mod pedals;

use crate::can::devices::motor_controller::MotorController;

use core::sync::atomic::Ordering;
use defmt::{println, trace};
use embassy_executor::Spawner;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let init::Resources {
        mut wdg,
        can_props,
        mut ticker,
    } = init::setup(spawner);
    let mut tick: u32 = 0;
    let mut log: u32 = 0;

    // Super loop
    loop {
        wdg.pet();
        ticker.next().await;

        // Constantly read and send torque to motor
        let torque = pedals::TORQUE.load(Ordering::Relaxed);
        MotorController.send_torque_cmd(torque);

        if tick.is_multiple_of(100) {
            println!("Torque: {}", torque);
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
