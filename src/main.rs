#![no_std]
#![no_main]

mod can;
mod clocks;
mod init;
mod pedals;

use crate::can::devices::motor_controller::MotorController;

use defmt::{println, trace};
use embassy_executor::Spawner;
use embassy_stm32::adc::SampleTime;

use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    let init::Resources {
        mut adc,
        mut pedal_pin,
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

        let adc_input = adc.blocking_read(&mut pedal_pin, SampleTime::CYCLES24_5);
        let torque = pedals::APPS1.percent(adc_input).map_or(0, pedals::torque);

        if MotorController.is_ready() {
            MotorController.send_torque_cmd(torque);
        } else {
            MotorController.send_stop_cmd();
        }

        if tick.is_multiple_of(100) {
            let adc_voltage = adc_input as u32 * 3300 / 4095;
            println!(
                "Pedal: {} raw, {} mV, {} torque",
                adc_input, adc_voltage, torque
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
