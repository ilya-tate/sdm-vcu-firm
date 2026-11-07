use defmt::{info, warn};
use embassy_stm32::can::Frame;
use embedded_can::Id::Standard as CanID;

pub(super) fn handle_frame(frame: &Frame) {
    // Get frame id
    let CanID(id) = *frame.id() else { return };
    let id: u16 = id.as_raw();

    // No payload -> Ignore frame
    if frame.header().rtr() {
        warn!("CAN Rx: Ignored 0x{:03x}, does not contain payload", id);
        return;
    }

    info!("CAN Rx: 0x{:03x} {:02x}", id, frame.data());
}
