use embassy_time::{Duration, Timer};

use super::queue_msg;

const FRAME_GAP: Duration = Duration::from_millis(100);

const FRAMES: &[(u16, &[u8])] = &[
    (0x000, &[]),
    (0x0A0, &[0x01, 0x02, 0x03]),
    (0x123, &[0xDE, 0xAD, 0xBE, 0xEF]),
    (0x7FF, &[0x00, 0x11, 0x22, 0x33, 0x44, 0x55, 0x66, 0x77]),
];

#[embassy_executor::task]
pub async fn send_test_frames() -> ! {
    for &(id, payload) in FRAMES {
        queue_msg(id, payload);
        Timer::after(FRAME_GAP).await;
    }

    loop {
        core::future::pending::<()>().await;
    }
}
