pub mod frame;
pub mod init;
pub mod msg_handler;

use defmt::{error, info, warn};
use embassy_stm32::Peri;
use embassy_stm32::can::{Can, CanRx, CanTx, OperatingMode};
use embassy_stm32::peripherals::{FDCAN1, PA11, PA12};
use embassy_time::{Duration, with_timeout};
use frame::{Frame, new_frame};

const TX_TIMEOUT: Duration = Duration::from_millis(5);

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

            Err(e) => error!("CAN Rx ERROR: {}", e),
        }
    }
}

pub async fn try_msg(tx: &mut CanTx<'_>, id: u16, payload: &[u8]) {
    if let Some(frame) = try_frame_build(id, payload) {
        if with_timeout(TX_TIMEOUT, tx.write(&frame)).await.is_err() {
            warn!("CAN Tx: 0x{:03x} timed out", id);
        }
    }
}
