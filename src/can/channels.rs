use embassy_stm32::can::{Frame, enums::BusError};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;

const CAPACITY: usize = 16;

pub static IN: Channel<ThreadModeRawMutex, Result<Frame, BusError>, CAPACITY> = Channel::new();
pub static OUT: Channel<ThreadModeRawMutex, Frame, CAPACITY> = Channel::new();
