// TODO:
// - Test if init hangs in can.rs or main.rs
// - Create checks for bus
// - Clear mailboxes after init
// - Add message handling
// - Wait for USB to connect to start

pub mod init;
pub mod frame;
pub mod message_handler;

use teensy4_bsp::{
    hal::iomuxc,
    pins::t41::{P0, P1},
    ral::{can, ccm},
};

pub const BITRATE: u32 = 500_000; // Default (entire car): 500kbps bitrate

pub struct CanBus {
    controller: can::CAN2,
    _rx: P0,
    _tx: P1,
}

impl CanBus {
    pub fn new(
        ccm: &mut ccm::CCM, mut rx: P0, mut tx: P1
    ) -> Result<Self, init::InitErr> {
        // Pins blocked frome external use
        let controller = unsafe { can::CAN2::instance() };

        init::clock_init(ccm);
        iomuxc::flexcan::prepare(&mut rx);
        iomuxc::flexcan::prepare(&mut tx);
        init::can_init(&controller);

        Ok(Self {
            controller, // Bus
            _rx: rx,    // Input
            _tx: tx,    // Output
        })
    }
}
