use core::sync::atomic::{AtomicBool, Ordering};
use defmt::{println, warn};
use embassy_stm32::can::Frame;

use super::Device;
use crate::can::get_frame_id;

static IS_FIRMWARE_PRINTED: AtomicBool = AtomicBool::new(false);
static IS_LOCKED_OUT: AtomicBool = AtomicBool::new(true);
static HAS_FAULT: AtomicBool = AtomicBool::new(false);

const CMD_ID: u16 = 0x0c0;
const PARAM_ID: u16 = 0x0c1;

// CMD WRITE STUFF

#[derive(Clone, Copy)]
enum Direction {
    // Reverse disallowed by rules,
    // otherweise would be 0x00
    Forward = 0x01,
}

#[derive(Clone, Copy)]
enum InverterMode {
    Disable = 0x00,
    Enable = 0x01,
    Discharge = 0x02,
    Speed = 0x04,
}

#[derive(Clone, Copy)]
struct CmdPayload {
    torque_cmd: u16,
    speed_cmd: u16,
    direction: Direction,
    inverter_mode: InverterMode,
    torque_limit: u16,
}

impl CmdPayload {
    const fn encode(&self) -> [u8; 8] {
        let torque_cmd = self.torque_cmd.to_le_bytes();
        let speed_cmd = self.speed_cmd.to_le_bytes();
        let torque_limit = self.torque_limit.to_le_bytes();

        [
            torque_cmd[0],
            torque_cmd[1],
            speed_cmd[0],
            speed_cmd[1],
            self.direction as u8,
            self.inverter_mode as u8,
            torque_limit[0],
            torque_limit[1],
        ]
    }
}

// PARAM STUFF

#[derive(Clone, Copy)]
enum ParamAddr {
    FaultClear = 0x14,
    InverterRunMode = 0x8e,
    InverterCmdMode = 0x8f,
    CanCmdMsgActive = 0x92,
    CanTimeout = 0xac,
}

#[derive(Clone, Copy)]
enum ParamMode {
    Read = 0x00,
    Write = 0x01,
}

#[derive(Clone, Copy)]
struct ParamPayload {
    address: ParamAddr,
    mode: ParamMode,
    reserved_head: u8,
    data: u16,
    reserved_tail: u16,
}

impl ParamPayload {
    const fn encode(&self) -> [u8; 8] {
        let address = (self.address as u16).to_le_bytes();
        let data = self.data.to_le_bytes();
        let reserved_tail = self.reserved_tail.to_le_bytes();

        [
            address[0],
            address[1],
            self.mode as u8,
            self.reserved_head,
            data[0],
            data[1],
            reserved_tail[0],
            reserved_tail[1],
        ]
    }

    const fn read(address: ParamAddr) -> Self {
        Self {
            address,
            mode: ParamMode::Read,
            reserved_head: 0x00,
            data: 0,
            reserved_tail: 0,
        }
    }

    const fn write(address: ParamAddr, data: u16) -> Self {
        Self {
            address,
            mode: ParamMode::Write,
            reserved_head: 0x00,
            data,
            reserved_tail: 0,
        }
    }
}

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

                IS_LOCKED_OUT.store(enable_flags & 0x80 != 0, Ordering::Relaxed);

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

                let post_faults = u32::from_le_bytes([
                    post_fault_lo_p0,
                    post_fault_lo_p1,
                    post_fault_hi_p0,
                    post_fault_hi_p1,
                ]);
                let run_faults = u32::from_le_bytes([
                    run_fault_lo_p0,
                    run_fault_lo_p1,
                    run_fault_hi_p0,
                    run_fault_hi_p1,
                ]);

                HAS_FAULT.store(post_faults != 0 || run_faults != 0, Ordering::Relaxed);

                println!(
                    "POST faults: 0x{:08x}\nRun faults: 0x{:08x}",
                    post_faults, run_faults
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
            // Command req
            0x0c0 => {
                let &[
                    torque_cmd_p0,
                    torque_cmd_p1,
                    speed_cmd_p0,
                    speed_cmd_p1,
                    direction,
                    inverter_mode,
                    torque_limit_p0,
                    torque_limit_p1,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete cmd payload");
                    return;
                };

                println!(
                    "Torque cmd: {}\nSpeed cmd: {}\nDirection: {}\nInverter mode: 0x{:02x}\nTorque limit: {}",
                    u16::from_le_bytes([torque_cmd_p0, torque_cmd_p1]),
                    u16::from_le_bytes([speed_cmd_p0, speed_cmd_p1]),
                    direction,
                    inverter_mode,
                    u16::from_le_bytes([torque_limit_p0, torque_limit_p1])
                );
            }
            // Param req
            0x0c1 => {
                let &[address_p0, address_p1, mode, _, data_p0, data_p1, _, _] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete param payload");
                    return;
                };

                println!(
                    "Param req: 0x{:02x}\nMode: {}\nData: {}",
                    u16::from_le_bytes([address_p0, address_p1]),
                    mode,
                    u16::from_le_bytes([data_p0, data_p1])
                );
            }
            // Param res
            0x0c2 => {
                let &[
                    address_p0,
                    address_p1,
                    write_success,
                    _,
                    data_p0,
                    data_p1,
                    _,
                    _,
                ] = frame.data()
                else {
                    warn!("Motor Controller: Incomplete param response payload");
                    return;
                };

                println!(
                    "Param res: 0x{:02x}\nWrite success: {}\nData: {}",
                    u16::from_le_bytes([address_p0, address_p1]),
                    write_success != 0,
                    u16::from_le_bytes([data_p0, data_p1])
                );
            }

            _ => {}
        }
    }
}

const FAULT_CLEAR: ParamPayload = ParamPayload::write(ParamAddr::FaultClear, 0);

const START: CmdPayload = CmdPayload {
    torque_cmd: 0,
    speed_cmd: 0,
    direction: Direction::Forward,
    inverter_mode: InverterMode::Enable,
    torque_limit: 0,
};

const STOP: CmdPayload = CmdPayload {
    torque_cmd: 0,
    speed_cmd: 0,
    direction: Direction::Forward,
    inverter_mode: InverterMode::Disable,
    torque_limit: 0,
};

impl MotorController {
    // Params

    pub fn clear_faults(&self) {
        self.write(PARAM_ID, &FAULT_CLEAR.encode());
    }

    pub fn read_config(&self) {
        use ParamAddr::{CanCmdMsgActive, CanTimeout, InverterCmdMode, InverterRunMode};
        const CONFIG_CHECKS: [ParamAddr; 4] = [
            InverterRunMode,
            InverterCmdMode,
            CanCmdMsgActive,
            CanTimeout,
        ];

        for address in CONFIG_CHECKS {
            self.write(PARAM_ID, &ParamPayload::read(address).encode());
        }
    }

    // Cmds
    pub fn send_start_cmd(&self) {
        self.write(CMD_ID, &START.encode());
    }

    pub fn send_stop_cmd(&self) {
        self.write(CMD_ID, &STOP.encode());
    }

    pub fn send_torque_cmd(&self, torque: u16) {
        // TODO: Add checks for direction, inverter mode,  and faults
        let payload = CmdPayload {
            torque_cmd: torque,
            speed_cmd: 0,
            direction: Direction::Forward,
            inverter_mode: InverterMode::Enable,
            torque_limit: 0,
        };
        self.write(CMD_ID, &payload.encode());
    }

    pub fn is_ready(&self) -> bool {
        !IS_LOCKED_OUT.load(Ordering::Relaxed) && !HAS_FAULT.load(Ordering::Relaxed)
    }
}
