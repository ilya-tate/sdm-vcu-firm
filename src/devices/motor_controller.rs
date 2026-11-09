use embassy_time::Duration;

use crate::can::{cmd::Cmd, queue_msg};

pub struct MotorController;

const FORWARD: u8 = 1;
const TEST_ID: u16 = 0x0c0;
const TEST_PAYLOAD: [u8; 8] = [0x14, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];
const PARAM_ID: u16 = 0x0c1;
const FAULT_CLEAR: [u8; 8] = [0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];

impl Cmd<8> for MotorController {
    const ID: u16 = 0x0c0;
    const INTERVAL: Duration = Duration::from_millis(100);

    fn payload(&mut self) -> [u8; 8] {
        TEST_PAYLOAD
    }
}

pub fn clear_faults() -> bool {
    queue_msg(PARAM_ID, &FAULT_CLEAR)
}
pub fn test() -> bool {
    queue_msg(0x0c1, &[0xeb, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00])
}

#[embassy_executor::task]
pub async fn send_cmds() -> ! {
    MotorController.send_cmds().await
}
