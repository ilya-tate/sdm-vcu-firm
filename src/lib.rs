//! # SDM VCU Firmware
//!
//! Firmware for the SDM vehicle control unit (STM32G4, Embassy).
//!
//! ## Modules
//! - [`can`]: FDCAN driver, message queues, and RX/TX tasks
//! - [`clocks`]: clock configuration and runtime verification
//! - [`wdg`]: watchdog reset detection

#![no_std]

use embassy_stm32::can::OperatingMode;

pub mod can;
pub mod clocks;
pub mod wdg;

/// Watchdog timeout in microseconds. Must outlast the expected main loop iteration time.
pub const WDG_TIMEOUT_US: u32 = 2_000_000;

/// CAN operating mode: internal loopback in debug builds, normal operation in release.
#[cfg(debug_assertions)]
pub const CAN_MODE: OperatingMode = OperatingMode::InternalLoopbackMode;
/// CAN operating mode: internal loopback in debug builds, normal operation in release.
#[cfg(not(debug_assertions))]
pub const CAN_MODE: OperatingMode = OperatingMode::NormalOperationMode;