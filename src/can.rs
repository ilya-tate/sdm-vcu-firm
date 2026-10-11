pub mod frame;
pub mod init;
pub mod msg_handler;

use defmt::{error, info};
use embassy_stm32::Peri;
use embassy_stm32::can::{Can, CanRx, CanTx, OperatingMode};
use embassy_stm32::peripherals::{FDCAN1, PA11, PA12};
use embassy_time::Timer;
use frame::{Frame, new_frame};

// 'd == driver
pub fn try_init<'d>(
    bus: Peri<'d, FDCAN1>,
    rx: Peri<'d, PA11>,
    tx: Peri<'d, PA12>,
    mode: OperatingMode,
) -> Option<Can<'d>> {
    match init::can_init(bus, rx, tx, mode) {
        Ok(can) => Some(can),

        Err(e) => {
            error!("CAN Init Error: {}", e);
            None
        }
    }
}

pub fn try_frame_build(id: u16, payload: &[u8]) -> Option<Frame> {
    match new_frame(id, payload) {
        Ok(frame) => {
            info!("CAN Frame: {}", frame);
            Some(frame)
        }

        Err(e) => {
            error!("Bad frame ERROR: {}", e);
            None
        }
    }
}

#[embassy_executor::task]
pub async fn input_listener(mut input: CanRx<'static>) {
    loop {
        match input.read().await {
            Ok(envelope) => msg_handler::handle_frame(&envelope.frame),

            // Bus errors repeat instantly
            Err(e) => {
                error!("CAN Rx ERROR: {}", e);
                // Avoid race conditions
                Timer::after_millis(10).await;
            }
        }
    }
}

pub async fn try_msg(tx: &mut CanTx<'_>, id: u16, payload: &[u8]) {
    if let Some(frame) = try_frame_build(id, payload) {
        tx.write(&frame).await;
    }
}
