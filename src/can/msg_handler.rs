use defmt::{info, trace, warn};
use embassy_stm32::can::Frame;
use embedded_can::Id::Standard as CanID;

pub const CAN_ID: u16 = 0x123; // Test value

pub(super) fn handle_frame(frame: &Frame) {
    // Get frame id
    let CanID(id) = *frame.id() else { return };
    let id: u16 = id.as_raw();

    // No payload -> Ignore frame
    if frame.header().rtr() {
        warn!("CAN Rx: Ignored 0x{:03x}, does not contain payload", id);
        return;
    }

    match id {
        CAN_ID => info!("CAN Rx: {:02x}, as input test", frame.data()),

        _ => trace!("CAN Rx: 0x{:03x} not handled", id),
    }
}
