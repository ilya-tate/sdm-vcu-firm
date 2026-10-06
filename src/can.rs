pub mod frame;

use defmt::{error, info};
use frame::{Frame, new_frame};

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
