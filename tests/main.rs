#![no_std]
#![no_main]

mod can_bus;
mod motor_controller;
mod pcb;
mod pedals;

use defmt_rtt as _;
