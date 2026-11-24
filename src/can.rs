pub mod devices;
pub mod init;
pub mod tasks;

use defmt::{error, println, warn};
use embassy_stm32::can::{
    Frame, Properties,
    enums::{BusError, BusErrorMode},
};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use embedded_can::Id;

// TODO: Store error messages separately

// Max messages that can be queued at once per queue
const CHANNEL_CAPACITY: usize = 16;

pub static RX_QUEUE: Channel<ThreadModeRawMutex, Result<Frame, BusError>, CHANNEL_CAPACITY> =
    Channel::new();
pub static TX_QUEUE: Channel<ThreadModeRawMutex, Frame, CHANNEL_CAPACITY> = Channel::new();

// To allow canFD frames to be logged later
pub fn get_frame_id(frame: &Frame) -> u32 {
    match frame.id() {
        Id::Standard(id) => id.as_raw().into(),
        Id::Extended(id) => id.as_raw(),
    }
}

pub fn queue_msg(id: u16, payload: &[u8]) -> bool {
    let Some(frame) = try_frame_build(id, payload) else {
        return false;
    };

    match TX_QUEUE.try_send(frame) {
        Ok(()) => true,

        Err(_) => {
            warn!(
                "CAN Tx ERROR: Queue full, dropping 0x{:03x}",
                get_frame_id(&frame)
            );
            false
        }
    }
}

pub fn try_frame_build(id: u16, payload: &[u8]) -> Option<Frame> {
    match Frame::new_standard(id, payload) {
        Ok(frame) => Some(frame),

        Err(e) => {
            error!("CAN Tx ERROR: Bad frame 0x{:03x},\n\t{}", id, e);
            None
        }
    }
}

pub fn is_valid_frame(frame: &Frame) -> bool {
    // Ensure standard 2.0 frame
    if matches!(frame.id(), Id::Extended(_)) {
        warn!(
            "CAN Rx: Ignored 0x{:08x}, canFD not supported",
            get_frame_id(frame)
        );

        return false;
    }

    // No payload -> Ignore frame
    if frame.header().rtr() {
        warn!(
            "CAN Rx: Ignored 0x{:03x}, does not contain payload",
            get_frame_id(frame)
        );

        return false;
    }

    return true;
}

pub fn log_bus_errs(props: &Properties) {
    let rx_errs = props.rx_error_count();
    let tx_errs = props.tx_error_count();

    match props.bus_error_mode() {
        BusErrorMode::ErrorActive => {
            println!("CAN Bus: ACTIVE\nErrors: {} Rx, {} Tx", rx_errs, tx_errs);
        }

        BusErrorMode::ErrorPassive => {
            warn!(
                "CAN Bus: ERROR passive\nErrors: {} Rx, {} Tx",
                rx_errs, tx_errs
            );
        }

        BusErrorMode::BusOff => {
            error!("CAN Bus: Off\nErrors: {} Rx, {} Tx", rx_errs, tx_errs);
        }
    }
}
