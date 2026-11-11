use core::sync::atomic::{AtomicBool, Ordering};
use defmt::{println, warn};
use embassy_stm32::can::Frame;

use super::Device;
use crate::can::get_frame_id;

static IS_FIRMWARE_PRINTED: AtomicBool = AtomicBool::new(false);

const CMD_ID: u16 = 0x0c0;
const PARAM_ID: u16 = 0x0c1;
const FAULT_CLEAR_PAYLOAD: [u8; 8] = [0x14, 0x00, 0x01, 0x00, 0x00, 0x00, 0x00, 0x00];
const TEST_PAYLOAD: [u8; 8] = [0x14, 0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00];

pub struct MotorController;

impl Device for MotorController {
    fn read(&self, frame: &Frame) {
        match get_frame_id(frame) {
            // Temperatures1
            0x0a0 => {
                let &[
                    module_a_temp_p0,
                    module_a_temp_p1,
                    module_b_temp_p0,
                    module_b_temp_p1,
                    module_c_temp_p0,
                    module_c_temp_p1,
                    gate_board_driver_temp_p0,
                    gate_board_driver_temp_p1,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete temperatures (1) payload");
                    return;
                };

                println!(
                    "Module A temp: {}\nModule B temp: {}\nModule C temp: {}\nGate Board Driver Temp: {}",
                    i16::from_le_bytes([module_a_temp_p0, module_a_temp_p1]),
                    i16::from_le_bytes([module_b_temp_p0, module_b_temp_p1]),
                    i16::from_le_bytes([module_c_temp_p0, module_c_temp_p1]),
                    i16::from_le_bytes([gate_board_driver_temp_p0, gate_board_driver_temp_p1])
                );
            }
            // Temperatures2
            0x0a1 => {
                let &[
                    control_board_temp_p0,
                    control_board_temp_p1,
                    rtd_one_temp_p0,
                    rtd_one_temp_p1,
                    rtd_two_temp_p0,
                    rtd_two_temp_p1,
                    rtd_three_temp_p0,
                    rtd_three_temp_p1,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete temperatures (2) payload");
                    return;
                };
                println!(
                    "Control board temp: {}\nRTD1 temp: {}\nRTD2 temp: {}\nRTD3/Stall Burst Model temp: {}",
                    i16::from_le_bytes([control_board_temp_p0, control_board_temp_p1]),
                    i16::from_le_bytes([rtd_one_temp_p0, rtd_one_temp_p1]),
                    i16::from_le_bytes([rtd_two_temp_p0, rtd_two_temp_p1]),
                    i16::from_le_bytes([rtd_three_temp_p0, rtd_three_temp_p1])
                );
            }
            // Temperatures3
            0x0a2 => {
                let &[
                    rtd_four_temp_p0,
                    rtd_four_temp_p1,
                    rtd_five_temp_p0,
                    rtd_five_temp_p1,
                    motor_temp_p0,
                    motor_temp_p1,
                    torque_shudder_p0,
                    torque_shudder_p1,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete temperatures (3) payload");
                    return;
                };
                println!(
                    "RTD4 temp: {}\nRTD5 temp: {}\nMotor temp: {}\nTorque shudder: {}",
                    i16::from_le_bytes([rtd_four_temp_p0, rtd_four_temp_p1]),
                    i16::from_le_bytes([rtd_five_temp_p0, rtd_five_temp_p1]),
                    i16::from_le_bytes([motor_temp_p0, motor_temp_p1]),
                    i16::from_le_bytes([torque_shudder_p0, torque_shudder_p1])
                );
            }
            // Analog Inputs
            0x0a3 => {}
            // Digital inputs
            0x0a4 => {}
            // Motor pos
            0x0a5 => {}
            // Currents info
            0x0a6 => {}
            // Voltages info
            0x0a7 => {
                let &[
                    dc_bus_voltage_p0,
                    dc_bus_voltage_p1,
                    output_voltage_p0,
                    output_voltage_p1,
                    vab_vd_voltage_p0,
                    vab_vd_voltage_p1,
                    vbc_vq_voltage_p0,
                    vbc_vq_voltage_p1,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete voltages payload");
                    return;
                };

                println!(
                    "DC bus voltage: {}\nOutput voltage: {}\nVAB/Vd voltage: {}\nVBC/Vq voltage: {}",
                    i16::from_le_bytes([dc_bus_voltage_p0, dc_bus_voltage_p1]),
                    i16::from_le_bytes([output_voltage_p0, output_voltage_p1]),
                    i16::from_le_bytes([vab_vd_voltage_p0, vab_vd_voltage_p1]),
                    i16::from_le_bytes([vbc_vq_voltage_p0, vbc_vq_voltage_p1])
                );
            }
            // Flux
            0x0a8 => {}
            // Internal voltages
            0x0a9 => {}
            // States
            0x0aa => {
                let &[
                    vsm_state,
                    _,
                    inverter_state,
                    _,
                    _,
                    _,
                    enable_flags,
                    direction_flags,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete states payload");
                    return;
                };

                println!(
                    "VSM state: {}\nInverter state: {}\nInverter enabled: {}\nEnable lockout: {}\nDirection: {}",
                    vsm_state,
                    inverter_state,
                    enable_flags & 0x01 != 0,
                    enable_flags & 0x80 != 0,
                    direction_flags & 0x01
                );
            }
            // Fault codes
            0x0ab => {
                let &[
                    post_fault_lo_p0,
                    post_fault_lo_p1,
                    post_fault_hi_p0,
                    post_fault_hi_p1,
                    run_fault_lo_p0,
                    run_fault_lo_p1,
                    run_fault_hi_p0,
                    run_fault_hi_p1,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete fault codes payload");
                    return;
                };

                println!(
                    "POST faults: 0x{:08x}\nRun faults: 0x{:08x}",
                    u32::from_le_bytes([
                        post_fault_lo_p0,
                        post_fault_lo_p1,
                        post_fault_hi_p0,
                        post_fault_hi_p1
                    ]),
                    u32::from_le_bytes([
                        run_fault_lo_p0,
                        run_fault_lo_p1,
                        run_fault_hi_p0,
                        run_fault_hi_p1
                    ])
                );
            }
            // Torque timer
            0x0ac => {}
            // Modulation flux weak
            0x0ad => {}
            // Firmware
            0x0ae => {
                let &[
                    eeprom_version_p0,
                    eeprom_version_p1,
                    software_version_p0,
                    software_version_p1,
                    date_month_day_p0,
                    date_month_day_p1,
                    date_year_p0,
                    date_year_p1,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete firmware info payload");
                    return;
                };
                // Only print once
                if IS_FIRMWARE_PRINTED.swap(true, Ordering::Relaxed) {
                    return;
                }

                println!(
                    "Motor Controller:
                    \n\tEEPROM: v{:x}
                    \n\tSoftware: v{:x}
                    \n\tDate: {:02x}/{:02x}/{:x}",
                    u16::from_le_bytes([eeprom_version_p0, eeprom_version_p1]),
                    u16::from_le_bytes([software_version_p0, software_version_p1]),
                    date_month_day_p1,
                    date_month_day_p0,
                    u16::from_le_bytes([date_year_p0, date_year_p1])
                );
            }
            // Diagnostics
            0x0af => {}
            // High speed
            0x0b0 => {}
            // Param res
            0x0c2 => {}

            _ => {}
        }
    }
}

impl MotorController {
    pub fn clear_faults(&self) {
        self.write(PARAM_ID, &FAULT_CLEAR_PAYLOAD);
    }

    pub fn test_write(&self) {
        self.write(CMD_ID, &TEST_PAYLOAD);
    }
}
