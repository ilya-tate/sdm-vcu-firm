//! Polled Classical CAN, standard data frames only. No software queue.
//! Mailbox 0 is reserved for the ERR005829 transmit workaround.
use teensy4_bsp::ral::{can, read_reg, write_reg};

use super::{CanBus, frame::Frame};

const TX: usize = 15;
const RX_EMPTY: u32 = 4 << 24;
const TX_INACTIVE: u32 = 8 << 24;
const TX_DATA: u32 = 12 << 24;

// The RAL omits mailbox RAM. Each Classical CAN mailbox is four u32 words
// starting at offset 0x80. All callers use mailbox 0..16 and word 0..4.
fn word(controller: &can::CAN2, mailbox: usize, index: usize) -> *mut u32 {
    assert!(mailbox < 16 && index < 4);
    let base: &can::RegisterBlock = controller;
    (base as *const _ as usize + 0x80 + mailbox * 16 + index * 4) as *mut u32
}

fn read(controller: &can::CAN2, mailbox: usize, index: usize) -> u32 {
    // SAFETY: word bounds-checks offsets into live CAN2 mailbox RAM.
    unsafe { word(controller, mailbox, index).read_volatile() }
}

fn write(controller: &can::CAN2, mailbox: usize, index: usize, value: u32) {
    // SAFETY: callers own CAN2; writes occur in freeze mode, to an inactive
    // TX mailbox, or to the reserved workaround mailbox.
    unsafe { word(controller, mailbox, index).write_volatile(value) }
}

pub(super) fn configure(controller: &can::CAN2) {
    write_reg!(can, controller, IMASK1, 0);
    write_reg!(can, controller, IMASK2, 0);
    for mailbox in 0..16 {
        for index in 0..4 {
            write(controller, mailbox, index, 0);
        }
        // Match standard data frames, accepting any 11-bit ID.
        controller.RXIMR[mailbox].write(0xc000_0000);
        write(
            controller,
            mailbox,
            0,
            if mailbox == 0 || mailbox == TX {
                TX_INACTIVE
            } else {
                RX_EMPTY
            },
        );
    }
    write_reg!(can, controller, IFLAG1, 0xffff);
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum TxError {
    Busy,
    BusOff,
}

#[derive(Debug)]
pub struct Received {
    pub frame: Frame,
    pub overrun: bool,
}

#[derive(Debug)]
pub enum RxError {
    UnsupportedFrame,
}

impl CanBus {
    /// Queue one frame. Success means queued, not acknowledged on the bus.
    pub fn try_transmit(&mut self, frame: &Frame) -> Result<(), TxError> {
        let controller = &self.controller;
        if read_reg!(can, controller, ESR1, FLTCONF) >= 2 {
            return Err(TxError::BusOff);
        }
        if read(controller, TX, 0) & (0xf << 24) != TX_INACTIVE {
            return Err(TxError::Busy);
        }
        write_reg!(can, controller, IFLAG1, 1 << TX);
        let words = frame.mailbox_words();
        write(controller, TX, 1, u32::from(frame.id()) << 18);
        write(controller, TX, 2, words[0]);
        write(controller, TX, 3, words[1]);
        write(controller, TX, 0, TX_DATA | ((frame.len() as u32) << 16));
        // NXP ERR005829: reserve the first mailbox and write INACTIVE twice.
        write(controller, 0, 0, TX_INACTIVE);
        write(controller, 0, 0, TX_INACTIVE);
        Ok(())
    }

    /// Consume the completion flag before queueing the next transmission.
    pub fn take_tx_complete(&mut self) -> bool {
        let controller = &self.controller;
        if read_reg!(can, controller, IFLAG1) & (1 << TX) == 0 {
            return false;
        }
        write_reg!(can, controller, IFLAG1, 1 << TX);
        true
    }

    /// Read one available frame. Mailbox order is not arrival order.
    pub fn try_receive(&mut self) -> Result<Option<Received>, RxError> {
        let controller = &self.controller;
        let flags = read_reg!(can, controller, IFLAG1);
        for mailbox in 1..TX {
            if flags & (1 << mailbox) == 0 {
                continue;
            }
            // Reading CS locks a full mailbox. A BUSY mailbox is retried later.
            let cs = read(controller, mailbox, 0);
            let code = (cs >> 24) & 0xf;
            if code != 2 && code != 6 {
                let _ = read_reg!(can, controller, TIMER);
                continue;
            }
            let id = read(controller, mailbox, 1);
            let words = [read(controller, mailbox, 2), read(controller, mailbox, 3)];
            // Clear only this W1C flag while locked, then unlock/rearm via TIMER.
            write_reg!(can, controller, IFLAG1, 1 << mailbox);
            let _ = read_reg!(can, controller, TIMER);
            if cs & ((1 << 21) | (1 << 20)) != 0 {
                return Err(RxError::UnsupportedFrame);
            }
            let frame = Frame::from_mailbox(id, cs >> 16, words);
            return Ok(Some(Received {
                frame,
                overrun: code == 6,
            }));
        }
        Ok(None)
    }

    /// Raw diagnostic snapshot (ESR1 error indications are cleared by reading).
    pub fn error_status(&self) -> (u32, u32) {
        let controller = &self.controller;
        (
            read_reg!(can, controller, ESR1),
            read_reg!(can, controller, ECR),
        )
    }
}
