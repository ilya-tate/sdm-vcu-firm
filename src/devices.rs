pub mod motor_controller;

use super::queue_msg;
use embassy_stm32::can::Frame;
use motor_controller::MotorController;

pub trait Device {
    fn read(&self, frame: &Frame);
    fn write(&self, id: u16, payload: &[u8]) {
        queue_msg(id, payload);
    }
}

const DEVICES: &[&dyn Device] = &[&MotorController];

pub fn forward_frame(frame: &Frame) {
    for device in DEVICES {
        device.read(frame);
    }
}
