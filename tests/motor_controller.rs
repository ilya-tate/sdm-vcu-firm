use sdm_vcu_firm::can::devices::motor_controller::MotorController;
use sdm_vcu_firm::can::devices::{self, Device};
use sdm_vcu_firm::pedals::torque;

use crate::can_bus::{check_tx, check_tx_empty, gen_frame};

pub const STATES_ID: u16 = 0x0aa;
pub const FAULTS_ID: u16 = 0x0ab;
pub const CMD_ID: u16 = 0x0c0;
pub const PARAM_ID: u16 = 0x0c1;
const START: [u8; 8] = [0, 0, 0, 0, 0x01, 0x01, 0, 0];
pub const STOP: [u8; 8] = [0, 0, 0, 0, 0x01, 0x00, 0, 0];
const LOCKOUT: [u8; 8] = [0, 0, 0, 0, 0, 0, 0x80, 0];
pub const FAULT: [u8; 8] = [0x01, 0, 0, 0, 0, 0, 0, 0];

fn ready_motor_controller() {
    MotorController.read(&gen_frame(STATES_ID, &[0; 8]));
    MotorController.read(&gen_frame(FAULTS_ID, &[0; 8]));
    assert!(MotorController.is_ready());
}

#[embedded_test::tests]
mod read {
    use super::*;

    #[test]
    fn sets_lockout_from_states() {
        ready_motor_controller();
        MotorController.read(&gen_frame(STATES_ID, &LOCKOUT));
        assert!(!MotorController.is_ready());
        MotorController.read(&gen_frame(STATES_ID, &[0; 8]));
        assert!(MotorController.is_ready());
    }

    #[test]
    fn ignores_other_states_bits() {
        ready_motor_controller();
        MotorController.read(&gen_frame(
            STATES_ID,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f, 0xff],
        ));
        assert!(MotorController.is_ready());
    }

    #[test]
    fn blocks_ready_on_run_fault() {
        ready_motor_controller();
        MotorController.read(&gen_frame(FAULTS_ID, &[0, 0, 0, 0, 0, 0, 0, 0x40]));
        assert!(!MotorController.is_ready());
        MotorController.read(&gen_frame(FAULTS_ID, &[0; 8]));
        assert!(MotorController.is_ready());
    }

    #[test]
    fn blocks_ready_on_post_fault() {
        ready_motor_controller();
        MotorController.read(&gen_frame(FAULTS_ID, &[0x01, 0, 0, 0, 0, 0, 0, 0]));
        assert!(!MotorController.is_ready());
        MotorController.read(&gen_frame(FAULTS_ID, &[0; 8]));
        assert!(MotorController.is_ready());
    }

    #[test]
    fn blocks_ready_on_every_fault_bit() {
        for byte in 0..8 {
            for bit in 0..8 {
                ready_motor_controller();
                let mut data = [0; 8];
                data[byte] = 1 << bit;
                MotorController.read(&gen_frame(FAULTS_ID, &data));
                assert!(!MotorController.is_ready());
            }
        }
    }

    #[test]
    fn ignores_short_states_payload() {
        for len in 0..8 {
            MotorController.read(&gen_frame(STATES_ID, &[0; 8][..len]));
            assert!(!MotorController.is_ready());
        }
        ready_motor_controller();
        for len in 0..8 {
            MotorController.read(&gen_frame(STATES_ID, &[0xff; 8][..len]));
            assert!(MotorController.is_ready());
        }
    }

    #[test]
    fn ignores_short_fault_payload() {
        ready_motor_controller();
        for len in 0..8 {
            MotorController.read(&gen_frame(FAULTS_ID, &[0xff; 8][..len]));
            assert!(MotorController.is_ready());
        }
        MotorController.read(&gen_frame(FAULTS_ID, &FAULT));
        for len in 0..8 {
            MotorController.read(&gen_frame(FAULTS_ID, &[0; 8][..len]));
            assert!(!MotorController.is_ready());
        }
    }

    #[test]
    fn ignores_unknown_id() {
        for id in [0x000, 0x09f, 0x0c3, 0x123, 0x7ff] {
            MotorController.read(&gen_frame(id, &[0; 8]));
            assert!(!MotorController.is_ready());
        }
        ready_motor_controller();
        for id in [0x000, 0x09f, 0x0c3, 0x123, 0x7ff] {
            MotorController.read(&gen_frame(id, &[0xff; 8]));
            assert!(MotorController.is_ready());
        }
        check_tx_empty();
    }

    #[test]
    fn keeps_ready_on_other_ids() {
        ready_motor_controller();
        for id in 0x0a0..=0x0c2 {
            if id == STATES_ID || id == FAULTS_ID {
                continue;
            }
            for len in 0..=8 {
                MotorController.read(&gen_frame(id, &[0xff; 8][..len]));
                assert!(MotorController.is_ready());
            }
        }
        check_tx_empty();
    }

    #[test]
    fn forward_frame_reaches_motor_controller() {
        devices::forward_frame(&gen_frame(STATES_ID, &[0; 8]));
        assert!(MotorController.is_ready());
    }
}

#[embedded_test::tests]
mod is_ready {
    use super::*;

    #[test]
    fn starts_not_ready() {
        assert!(!MotorController.is_ready());
    }

    #[test]
    fn needs_states_frame() {
        MotorController.read(&gen_frame(FAULTS_ID, &[0; 8]));
        assert!(!MotorController.is_ready());
    }

    #[test]
    fn needs_lockout_and_fault_cleared() {
        MotorController.read(&gen_frame(STATES_ID, &LOCKOUT));
        MotorController.read(&gen_frame(FAULTS_ID, &FAULT));
        assert!(!MotorController.is_ready());

        MotorController.read(&gen_frame(STATES_ID, &[0; 8]));
        assert!(!MotorController.is_ready());

        MotorController.read(&gen_frame(STATES_ID, &LOCKOUT));
        MotorController.read(&gen_frame(FAULTS_ID, &[0; 8]));
        assert!(!MotorController.is_ready());

        MotorController.read(&gen_frame(STATES_ID, &[0; 8]));
        assert!(MotorController.is_ready());
    }
}

#[embedded_test::tests]
mod clear_faults {
    use super::*;

    #[test]
    fn sends_fault_clear() {
        MotorController.clear_faults();
        check_tx(PARAM_ID, [0x14, 0, 0x01, 0, 0, 0, 0, 0]);
        check_tx_empty();
    }

    #[test]
    fn waits_for_fault_frame() {
        ready_motor_controller();
        MotorController.read(&gen_frame(FAULTS_ID, &FAULT));
        MotorController.clear_faults();
        assert!(!MotorController.is_ready());
        MotorController.read(&gen_frame(FAULTS_ID, &[0; 8]));
        assert!(MotorController.is_ready());
    }
}

#[embedded_test::tests]
mod commands {
    use super::*;

    #[test]
    fn sends_stop_when_not_ready() {
        MotorController.send_torque_cmd(500);
        check_tx(CMD_ID, STOP);
        check_tx_empty();
    }

    #[test]
    fn sends_torque() {
        ready_motor_controller();
        MotorController.send_torque_cmd(500);
        check_tx(CMD_ID, [0xf4, 0x01, 0, 0, 0x01, 0x01, 0, 0]);
        check_tx_empty();
    }

    #[test]
    fn sends_zero_torque() {
        ready_motor_controller();
        MotorController.send_torque_cmd(0);
        check_tx(CMD_ID, START);
        check_tx_empty();
    }

    #[test]
    fn sends_max_torque() {
        ready_motor_controller();
        MotorController.send_torque_cmd(torque(100));
        check_tx(CMD_ID, [0xe8, 0x03, 0, 0, 0x01, 0x01, 0, 0]);
        check_tx_empty();
    }

    #[test]
    fn read_config_requests_each_param() {
        MotorController.read_config();
        check_tx(PARAM_ID, [0x8e, 0, 0, 0, 0, 0, 0, 0]);
        check_tx(PARAM_ID, [0x8f, 0, 0, 0, 0, 0, 0, 0]);
        check_tx(PARAM_ID, [0x92, 0, 0, 0, 0, 0, 0, 0]);
        check_tx(PARAM_ID, [0xac, 0, 0, 0, 0, 0, 0, 0]);
        check_tx_empty();
    }

    #[test]
    fn send_start_cmd_sends_start() {
        MotorController.send_start_cmd();
        check_tx(CMD_ID, START);
        check_tx_empty();
    }

    #[test]
    fn send_stop_cmd_sends_stop() {
        MotorController.send_stop_cmd();
        check_tx(CMD_ID, STOP);
        check_tx_empty();
    }
}
