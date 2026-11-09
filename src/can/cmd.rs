use embassy_time::{Duration, Ticker};

use super::queue_msg;

pub trait Cmd<const N: usize> {
    const ID: u16;
    const INTERVAL: Duration;

    fn payload(&mut self) -> [u8; N];

    async fn send_cmds(&mut self) -> ! {
        let mut ticker = Ticker::every(Self::INTERVAL);

        loop {
            queue_msg(Self::ID, &self.payload());
            ticker.next().await;
        }
    }
}
