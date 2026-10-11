pub use embassy_stm32::can::enums::FrameCreateError::{self, InvalidCanId, InvalidDataLength};
pub use embassy_stm32::can::frame::Frame;

// canFD not in use, using can2.0 max
pub const ID_MAX: u16 = 0x7ff; // 0000 0111 1111 1111
// can2.0 payload max is 8 bytes
pub const LEN_MAX: usize = 8; // Functionally u4

pub fn new_frame(id: u16, payload: &[u8]) -> Result<Frame, FrameCreateError> {
    if id > ID_MAX {
        return Err(InvalidCanId);
    }
    if payload.len() > LEN_MAX {
        return Err(InvalidDataLength);
    }

    // Gen new embassy can2.0
    Frame::new_standard(id, payload)
}
