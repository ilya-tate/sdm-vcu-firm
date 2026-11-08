pub mod channels;
pub mod init;
#[cfg(debug_assertions)]
pub mod test_frames;

use defmt::{error, trace, warn};
use embassy_stm32::can::{
    CanRx, CanTx, Frame, Properties,
    enums::{BusError, BusErrorMode},
};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};
use embassy_time::{Duration, TimeoutError, Timer, with_timeout};
use embedded_can::Id;

// TODO: Store error messages separately

const TX_TIMEOUT: Duration = Duration::from_millis(5);

// Copies of frame from build
struct Msg<'a> {
    id: u32,
    payload: &'a [u8],
}

// To allow canFD frames to be logged later
fn get_frame_id(frame: &Frame) -> u32 {
    match frame.id() {
        Id::Standard(id) => id.as_raw().into(),
        Id::Extended(id) => id.as_raw(),
    }
}

pub fn queue_msg(id: u16, payload: &[u8]) -> bool {
    let Some(frame) = try_frame_build(id, payload) else {
        return false;
    };

    match channels::OUT.try_send(frame) {
        Ok(()) => {
            trace!("CAN Tx: Queued 0x{:03x}", get_frame_id(&frame));
            true
        }

        Err(_) => {
            warn!(
                "CAN Tx ERROR: Queue full, dropping 0x{:03x}",
                get_frame_id(&frame)
            );
            false
        }
    }
}

fn try_frame_build(id: u16, payload: &[u8]) -> Option<Frame> {
    match Frame::new_standard(id, payload) {
        Ok(frame) => {
            trace!("CAN Tx: Built frame 0x{:03x} {:02x}", id, payload);
            Some(frame)
        }

        Err(e) => {
            error!("CAN Tx ERROR: Bad frame 0x{:03x},\n\t{}", id, e);
            None
        }
    }
}

fn try_msg_build(frame: &Frame) -> Option<Msg<'_>> {
    // Get frame id if not FD
    if matches!(frame.id(), Id::Extended(_)) {
        warn!(
            "CAN Rx: Ignored 0x{:08x}, canFD not supported",
            get_frame_id(frame)
        );
        return None;
    }
    let id: u32 = get_frame_id(frame);

    // No payload -> Ignore frame
    if frame.header().rtr() {
        warn!("CAN Rx: Ignored 0x{:03x}, does not contain payload", id);
        return None;
    }
    let payload = frame.data();

    trace!("CAN Rx: Building 0x{:03x}", id);
    Some(Msg { id, payload })
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
            Ok(Some(frame)) => {
                if requeue.try_send(frame).is_err() {
                    warn!(
                        "CAN Tx ERROR: Queue full, dropping 0x{:03x}",
                        get_frame_id(&frame)
                    );
                }
            }

            Err(TimeoutError) => warn!(
                "CAN Tx ERROR: Timeout, dropping 0x{:03x}",
                get_frame_id(&frame)
            ),
        }
    }
}

#[embassy_executor::task]
pub async fn input_listener(
    mut rx_bus: CanRx<'static>,
    queue: DynamicSender<'static, Result<Frame, BusError>>,
) -> ! {
    loop {
        match rx_bus.read().await {
            Ok(envelope) => {
                let frame = envelope.frame;
                if queue.try_send(Ok(frame)).is_err() {
                    warn!(
                        "CAN Rx ERROR: Queue full, dropping 0x{:03x}",
                        get_frame_id(&frame)
                    );
                }
            }

            Err(e) => {
                // Skip errors to maintian frame priority
                let _ = queue.try_send(Err(e));
                // Stops instantly repeating errors
                Timer::after_millis(10).await;
            }
        }
    }
}

#[embassy_executor::task]
pub async fn input_processor(queue: DynamicReceiver<'static, Result<Frame, BusError>>) -> ! {
    loop {
        match queue.receive().await {
            Ok(frame) => {
                if let Some(msg) = try_msg_build(&frame) {
                    trace!("CAN Rx: Built frame, 0x{:03x} {:02x}", msg.id, msg.payload)
                }
            }

            Err(e) => warn!("CAN Rx ERROR: Bus error, {}", e),
        }
    }
}

pub fn log_bus_errs(props: &Properties) {
    let rx_errs = props.rx_error_count();
    let tx_errs = props.tx_error_count();

    match props.bus_error_mode() {
        BusErrorMode::ErrorActive => {
            trace!(
                "CAN Bus: ERROR active\n\t\t Errors: {} Rx, {} Tx",
                rx_errs, tx_errs
            );
        }

        BusErrorMode::ErrorPassive => {
            warn!(
                "CAN Bus: ERROR passive\n\t\t Errors: {} Rx, {} Tx",
                rx_errs, tx_errs
            );
        }

        BusErrorMode::BusOff => {
            error!("CAN Bus: Off\n\t\t Errors: {} Rx, {} Tx", rx_errs, tx_errs);
        }
    }
}
