use defmt::warn;
use embassy_stm32::can::{CanRx, CanTx, Frame, enums::BusError};
use embassy_sync::channel::{DynamicReceiver, DynamicSender};
use embassy_time::{Duration, TimeoutError, with_timeout};

use super::{devices, get_frame_id, is_valid_frame};

const TX_TIMEOUT: Duration = Duration::from_millis(5);

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
                warn!("Can Rx: Error, {}", e);
                // Skip errors to maintian frame priority
                let _ = queue.try_send(Err(e));
                // Stops instantly repeating errors
                //Timer::after_millis(10).await;
            }
        }
    }
}

#[embassy_executor::task]
pub async fn input_processor(queue: DynamicReceiver<'static, Result<Frame, BusError>>) -> ! {
    loop {
        match queue.receive().await {
            Ok(frame) => {
                if is_valid_frame(&frame) {
                    devices::forward_frame(&frame);
                }
            }

            Err(e) => warn!("CAN Rx ERROR: Bus error, {}", e),
        }
    }
}
