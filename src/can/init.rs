use core::ops::Range;

use teensy4_bsp::ral::{can, ccm, modify_reg, read_reg, write_reg};

use super::BITRATE;

// Can init params
const CAN_CLOCK: u32 = 24_000_000; // Can clock: 24Mhz
const BIT_QUANT: u32 = 16; // Time quant per transfer bit
const PRESCALE_DIVIDE: u32 = CAN_CLOCK / (BITRATE * BIT_QUANT) - 1;
const MAILBOXES: Range<usize> = 0..16; // Possible mailbox lenghts
const MAILBOX_DATA: usize = 4; // controller, id, bytes 0-3, bytes 4-7
const LOC_MAILBOX_HEAD: usize = 0x80 / 4;
const LOC_RX_START: usize = 0x880 / 4;
// Time to wait before error thrown,
// addressing hanging inits
const WAIT_TIMEOUT: u32 = 1_000_000;

// Divides bitrate evenly across can clock
const _: () = assert!(CAN_CLOCK % (BITRATE * BIT_QUANT) == 0);

// Waits for mcr bit,
// returns errors when not found
macro_rules! wait_mcr {
    (
        $controller: expr,
        $field:ident == $val:literal,
        $stage:ident
    ) => {
        try_until_done(InitErr::$stage, || read_reg!(can, $controller, MCR, $field == $val))?
    };
}

pub(super) fn can_init(controller: &can::CAN2) -> Result<(), InitErr> {
    modify_reg!(can, controller, MCR, MDIS: 0); // Enable
    wait_mcr!(controller, LPMACK == 0, LowPowerErr); // Wait not low power

    modify_reg!(can, controller, MCR, SOFTRST: 1); // Reset reg
    wait_mcr!(controller, SOFTRST == 0, SoftResetUncleared); // Wait done

    // Allows config regs to be writable
    modify_reg!(can, controller, MCR, FRZ: 1, HALT: 1 ); // Freeze
    wait_mcr!(controller, FRZACK == 1, FreezeUnset); // Wait

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
    wait_mcr!(controller, FRZACK == 0, FreezeStopUncleared);
    wait_mcr!(controller, NOTRDY == 0, NotReadyUncleared); // Requires synced bus

    Ok(())
}

pub(super) fn clock_init(clocks: &ccm::CCM) {
    // Block gate during reg modification
    modify_reg!(ccm, clocks, CCGR0, CG9: 0, CG10: 0);
    // Matches can clock speed
    modify_reg!(ccm, clocks, CSCMR2, CAN_CLK_SEL: 1, CAN_CLK_PODF: 0);
    // Releast gate
    modify_reg!(ccm, clocks, CCGR0, CG9: 0b11, CG10: 0b11);
}

#[derive(Copy, Clone, Debug)]
pub enum InitErr {
    // Errors thrown when param left unlceared
    // Addresses hanging init
    LowPowerErr,            // LPMACK
    SoftResetUncleared,     // SOFTRST
    FreezeUnset,            // FRZACK unset
    FreezeStopUncleared,    // FRZACK
    NotReadyUncleared,      // NOTRDY
}

// Tries until wait time is over,
// throws on timeout
fn try_until_done(
    E: InitErr, mut done: impl FnMut() -> bool
) -> Result<(), InitErr> {
    for _i in 0..WAIT_TIMEOUT {
        if done() {
            return Ok(());
        }
    }

    Err(E)
}

