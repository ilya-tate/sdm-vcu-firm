pub mod init;
pub mod msg_handler;

use defmt::{error, info, warn};
use embassy_stm32::can::{CanRx, CanTx, Frame};
use embassy_time::{Duration, with_timeout};

const TX_TIMEOUT: Duration = Duration::from_millis(5);

pub fn try_frame_build(id: u16, payload: &[u8]) -> Option<Frame> {
    match Frame::new_standard(id, payload) {
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
