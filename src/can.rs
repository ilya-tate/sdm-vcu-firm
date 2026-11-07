pub mod channels;
pub mod init;
#[cfg(debug_assertions)]
pub mod test_frames;

use defmt::{debug, error, info, trace, warn};
use embassy_stm32::can::{CanRx, CanTx, Frame, Properties, enums::BusErrorMode};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};
use embassy_time::{Duration, TimeoutError, with_timeout};
use embedded_can::Id::Standard as CanID;

const TX_TIMEOUT: Duration = Duration::from_millis(5);

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

fn handle_frame(frame: &Frame) {
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

#[embassy_executor::task]
pub async fn input_processor(queue: DynamicReceiver<'static, Frame>) -> ! {
    loop {
        let frame = queue.receive().await;
        msg_handler::handle_frame(&frame);
    }
}

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
