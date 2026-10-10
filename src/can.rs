pub mod frame;
pub mod init;

use defmt::{error, info};
use embassy_stm32::Peri;
use embassy_stm32::can::{Can, OperatingMode};
use embassy_stm32::peripherals::{FDCAN1, PA11, PA12};
use frame::{Frame, new_frame};

// 'd == driver
pub fn try_can_init<'d>(
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
