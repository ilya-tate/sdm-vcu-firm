// TODO:
// - Test if init hangs in can.rs or main.rs
// - Create checks for bus
// - Clear mailboxes after init
// - Add message handling

pub mod frame;
//pub mod message_handler;

use core::ops::Range;

use teensy4_bsp::{
    hal::iomuxc,
    pins::t41::{P0, P1},
    ral::{can, ccm, modify_reg, read_reg, write_reg},
};

pub const BITRATE: u32 = 500_000; // Default (entire car): 500kbps bitrate
// Can init params
const CAN_CLOCK: u32 = 24_000_000; // Can clock: 24Mhz
const BIT_QUANT: u32 = 16; // Time quant per transfer bit
const PRESCALE_DIVIDE: u32 = CAN_CLOCK / (BITRATE * BIT_QUANT) - 1;
const MAILBOXES: Range<usize> = 0..16; // Possible mailbox lenghts
const MAILBOX_DATA: usize = 4; // controller, id, bytes 0-3, bytes 4-7
const LOC_MAILBOX_HEAD: usize = 0x80 / 4;
const LOC_RX_START: usize = 0x880 / 4;

// Divides bitrate evenly across can clock
const _: () = assert!(CAN_CLOCK % (BITRATE * BIT_QUANT) == 0);

pub struct CanBus {
    controller: can::CAN2,
    _rx: P0,
    _tx: P1,
}

impl CanBus {
    pub fn new(ccm: &mut ccm::CCM, mut rx: P0, mut tx: P1) -> Self {
        // Pins blocked frome external use
        let controller = unsafe { can::CAN2::instance() };

        clock_init(ccm);
        iomuxc::flexcan::prepare(&mut rx);
        iomuxc::flexcan::prepare(&mut tx);
        can_init(&controller);

        Self {
            controller, // Bus
            _rx: rx,    // Input
            _tx: tx,    // Output
        }
    }
}

fn can_init(controller: &can::CAN2) {
    modify_reg!(can, controller, MCR, MDIS: 0); // Enable
    while read_reg!(can, controller, MCR, LPMACK == 1) {} // Wait not low power

    modify_reg!(can, controller, MCR, SOFTRST: 1); // Reset reg
    while read_reg!(can, controller, MCR, SOFTRST == 1) {} // Wait done

    // Allows config regs to be writable
    modify_reg!(can, controller, MCR, FRZ: 1, HALT: 1 ); // Freeze
    while read_reg!(can, controller, MCR, FRZACK == 0) {} // Wait

    modify_reg!(can, controller, MCR,
        SRXDIS: 1,  // Reject frames from self
        IRMQ: 1,    // Rx for each mailbox
        AEN: 1,     // Allow abort output reqs
        MAXMB: MAILBOXES.end as u32 - 1
    );

    // Control timings
    write_reg!(can, controller, CTRL1,
        // 24Mhz / 3 => 8Mhz
        // 125 ns (0.000125 ms) time quant
        // Per bit: 1 (sync) + 6 + 7 + 2
        PRESDIV: PRESCALE_DIVIDE,
        PROPSEG: 5, // 6
        PSEG1: 6,   // 7
        PSEG2: 1,   // 2
        RJW: 1      // resync jump
    );

    // Mailbox ram at random loc on reset
    let mem_block: &can::RegisterBlock = controller;
    let mem_ptr = mem_block as *const can::RegisterBlock as *mut u32;

    for mailbox in MAILBOXES {
        // Clear all to inactive, no id, no data
        for i in 0..MAILBOX_DATA {
            unsafe {
                // Ensures mailbox data, is in ram, and frozen
                mem_ptr
                    .add(LOC_MAILBOX_HEAD + mailbox * MAILBOX_DATA + i)
                    .write_volatile(0)
            };
        }

        // Clear input mask (per mailbox)
        unsafe { mem_ptr.add(LOC_RX_START + mailbox).write_volatile(0) };
    }

    // Freezed until sync
    modify_reg!(can, controller, MCR, FRZ: 0, HALT: 0);
    while read_reg!(can, controller, MCR, FRZACK == 1) {}
    while read_reg!(can, controller, MCR, NOTRDY == 1) {}
}

fn clock_init(clocks: &ccm::CCM) {
    // Block gate during reg modification
    modify_reg!(ccm, clocks, CCGR0, CG9: 0, CG10: 0);
    // Matches can clock speed
    modify_reg!(ccm, clocks, CSCMR2, CAN_CLK_SEL: 1, CAN_CLK_PODF: 0);
    // Releast gate
    modify_reg!(ccm, clocks, CCGR0, CG9: 0b11, CG10: 0b11);
}
