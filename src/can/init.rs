use teensy4_bsp::ral::{can, ccm, modify_reg, read_reg, write_reg};

use super::{BITRATE, message_handler};

// Can init params
const CAN_CLOCK: u32 = 24_000_000; // Can clock: 24Mhz
const BIT_QUANT: u32 = 16; // Time quant per transfer bit
const PRESCALE_DIVIDE: u32 = CAN_CLOCK / (BITRATE * BIT_QUANT) - 1;
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
        try_until_done(InitErr::$stage, || {
            read_reg!(can, $controller, MCR, $field == $val)
        })?
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
        MAXMB: 15
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

    // Compare IDE/RTR as well as identifiers; never auto-answer remote frames.
    modify_reg!(can, controller, CTRL2, EACEN: 1, RRS: 1);
    message_handler::configure(controller);

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
    LowPowerErr,         // LPMACK
    SoftResetUncleared,  // SOFTRST
    FreezeUnset,         // FRZACK unset
    FreezeStopUncleared, // FRZACK
    NotReadyUncleared,   // NOTRDY
}

// Tries until wait time is over,
// throws on timeout
fn try_until_done(e: InitErr, mut done: impl FnMut() -> bool) -> Result<(), InitErr> {
    for _i in 0..WAIT_TIMEOUT {
        if done() {
            return Ok(());
        }
    }

    Err(e)
}
