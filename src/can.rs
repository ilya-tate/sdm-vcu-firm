pub mod channels;
pub mod init;
pub mod msg_handler;

use defmt::{debug, error, trace, warn};
use embassy_stm32::can::{CanRx, CanTx, Frame, Properties, enums::BusErrorMode};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};
use embassy_time::{Duration, TimeoutError, with_timeout};

pub const CAN_ID: u16 = 0x123; // Test value

const TX_TIMEOUT: Duration = Duration::from_millis(5);

/// Queues a CAN message for transmission. Returns true if the message was successfully queued, false otherwise.
pub fn queue_msg(id: u16, payload: &[u8]) -> bool {
    let Some(frame) = try_frame_build(id, payload) else {
        return false;
    };

    match channels::OUT.try_send(frame) {
        Ok(()) => {
            trace!("CAN Tx: Queued {}", frame);
            true
        }

        Err(_) => {
            warn!("CAN Tx ERROR: Dropping {}, queue full", frame);
            false
        }
    }
}

/// Attempts to build a CAN frame with the given ID and payload. Returns Some(Frame) if successful, None otherwise.
fn try_frame_build(id: u16, payload: &[u8]) -> Option<Frame> {
    match Frame::new_standard(id, payload) {
        Ok(frame) => {
            trace!("CAN Tx: Built {}", frame);
            Some(frame)
        }

        Err(e) => {
            error!("CAN Tx ERROR: Bad frame for {}, {}", id, e);
            None
        }
    }
}

/// Spawns tasks for sending and receiving CAN messages, as well as processing received messages.
#[embassy_executor::task]
pub async fn output_sender(
    mut tx_bus: CanTx<'static>,
    queue: DynamicReceiver<'static, Frame>,
    requeue: DynamicSender<'static, Frame>,
) -> ! {
    loop {
        let frame = queue.receive().await;

        match with_timeout(TX_TIMEOUT, tx_bus.write(&frame)).await {
            Ok(None) => {}

            // Make room for higher priority frames
            Ok(Some(bumped)) => {
                if requeue.try_send(bumped).is_err() {
                    warn!("CAN Tx ERROR: Dropping bumped {}, queue full", bumped);
                }
            }

            Err(TimeoutError) => warn!("CAN Tx ERROR: Dropping {}, timed out", frame),
        }
    }
}

/// Spawns a task for listening to incoming CAN messages and forwarding them to a queue for processing.
#[embassy_executor::task]
pub async fn input_listener(mut rx_bus: CanRx<'static>, queue: DynamicSender<'static, Frame>) -> ! {
    loop {
        match rx_bus.read().await {
            Ok(envelope) => {
                if queue.try_send(envelope.frame).is_err() {
                    warn!("CAN Rx ERROR: Dropping {}, queue full", envelope.frame);
                }
            }

            Err(e) => debug!("CAN Rx ERROR: Bus error, {}", e),
        }
    }
}

/// Spawns a task for processing incoming CAN messages from a queue.
#[embassy_executor::task]
pub async fn input_processor(queue: DynamicReceiver<'static, Frame>) -> ! {
    loop {
        let frame = queue.receive().await;
        msg_handler::handle_frame(&frame);
    }
}

/// Logs the current CAN bus error counts and mode, providing insight into the health of the CAN bus.
pub fn log_bus_errs(props: &Properties) {
    let rx_errs = props.rx_error_count();
    let tx_errs = props.tx_error_count();

    match props.bus_error_mode() {
        BusErrorMode::ErrorActive => {
            trace!("CAN Errors Active: {} Rx, {} Tx", rx_errs, tx_errs);
        }

        BusErrorMode::ErrorPassive => {
            warn!("CAN Errors Passive: {} Rx, {} Tx", rx_errs, tx_errs);
        }

        BusErrorMode::BusOff => {
            error!("CAN Bus Off: {} Rx, {} Tx", rx_errs, tx_errs);
        }
    }
}
