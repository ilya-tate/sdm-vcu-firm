#![no_std]
#![no_main]

use defmt::assert_eq;
use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::can::{
    Can, Frame, OperatingMode,
    enums::{BusError, BusErrorMode},
};
use embassy_stm32::pac::RCC;
use embassy_stm32::rcc::{APBPrescaler, Sysclk, mux};
use embassy_time::{Duration, Timer, block_for};
use embedded_can::StandardId;

use sdm_vcu_firm::can::devices::Device;
use sdm_vcu_firm::can::devices::motor_controller::MotorController;
use sdm_vcu_firm::can::{
    self, RX_QUEUE, TX_QUEUE, devices, get_frame_id, is_valid_frame, queue_msg, try_frame_build,
};
use sdm_vcu_firm::pedals::{APPS1, torque};
use sdm_vcu_firm::{clocks, init};

const STATES_ID: u16 = 0x0aa;
const FAULTS_ID: u16 = 0x0ab;
const CMD_ID: u16 = 0x0c0;
const PARAM_ID: u16 = 0x0c1;
const START: [u8; 8] = [0, 0, 0, 0, 0x01, 0x01, 0, 0];
const STOP: [u8; 8] = [0, 0, 0, 0, 0x01, 0x00, 0, 0];
const LOCKOUT: [u8; 8] = [0, 0, 0, 0, 0, 0, 0x80, 0];
const FAULT: [u8; 8] = [0x01, 0, 0, 0, 0, 0, 0, 0];

fn gen_frame(id: u16, data: &[u8]) -> Frame {
    Frame::new_standard(id, data).unwrap()
}

fn gen_extended(id: u32, data: &[u8]) -> Frame {
    Frame::new_extended(id, data).unwrap()
}

fn gen_remote(id: u16, len: usize) -> Frame {
    Frame::new_remote(StandardId::new(id).unwrap(), len).unwrap()
}

fn check_tx(id: u16, data: [u8; 8]) {
    let frame = TX_QUEUE.try_receive().unwrap();
    assert_eq!(get_frame_id(&frame), id as u32);
    assert_eq!(frame.data(), &data[..]);
}

fn check_tx_empty() {
    assert!(TX_QUEUE.try_receive().is_err());
}

fn fill_tx() {
    for _ in 0..TX_QUEUE.capacity() {
        assert!(queue_msg(0x123, &[0; 8]));
    }
}

fn ready_motor_controller() {
    MotorController.read(&gen_frame(STATES_ID, &[0; 8]));
    MotorController.read(&gen_frame(FAULTS_ID, &[0; 8]));
    assert!(MotorController.is_ready());
}

fn start_loopback() -> Can<'static> {
    let peripherals = embassy_stm32::init(clocks::config());
    can::init::can_init(
        peripherals.FDCAN1,
        peripherals.PA11,
        peripherals.PA12,
        OperatingMode::InternalLoopbackMode,
    )
    .unwrap()
}

mod can_bus {
    #[embedded_test::tests]
    mod get_frame_id {
        use crate::*;
        use defmt::assert_eq;

        #[test]
        fn reads_standard() {
            for id in [0x000, 0x0c0, 0x7ff] {
                assert_eq!(get_frame_id(&gen_frame(id, &[])), id as u32);
            }
        }

        #[test]
        fn reads_extended() {
            for id in [0x0000000, 0x00000c0, 0x1234567, 0x1fffffff] {
                assert_eq!(get_frame_id(&gen_extended(id, &[0x01])), id);
            }
        }
    }

    #[embedded_test::tests]
    mod try_frame_build {
        use crate::*;
        use defmt::assert_eq;

        #[test]
        fn builds_valid_frame() {
            let frame = try_frame_build(0x0c1, &[1, 2, 3, 4, 5, 6, 7, 8]).unwrap();
            assert_eq!(get_frame_id(&frame), 0x0c1);
            assert_eq!(frame.data(), &[1, 2, 3, 4, 5, 6, 7, 8][..]);
            assert!(is_valid_frame(&frame));
        }

        #[test]
        fn builds_every_payload_length() {
            let payload = [1, 2, 3, 4, 5, 6, 7, 8];
            for len in 0..=8 {
                let frame = try_frame_build(0x0c1, &payload[..len]).unwrap();
                assert_eq!(frame.data(), &payload[..len]);
            }
        }

        #[test]
        fn builds_min_and_max_id() {
            for id in [0x000, 0x7ff] {
                let frame = try_frame_build(id, &[0; 8]).unwrap();
                assert_eq!(get_frame_id(&frame), id as u32);
            }
        }

        #[test]
        fn rejects_long_payload() {
            assert!(try_frame_build(0x0c1, &[0; 9]).is_none());
            assert!(try_frame_build(0x0c1, &[0; 64]).is_none());
        }

        #[test]
        fn rejects_payload_length_overflow() {
            assert!(try_frame_build(0x0c1, &[0; 256]).is_none());
            assert!(try_frame_build(0x0c1, &[0; 264]).is_none());
        }

        #[test]
        fn rejects_bad_id() {
            assert!(try_frame_build(0x800, &[0; 8]).is_none());
            assert!(try_frame_build(u16::MAX, &[0; 8]).is_none());
        }
    }

    #[embedded_test::tests]
    mod is_valid_frame {
        use crate::*;

        #[test]
        fn accepts_standard_data() {
            for id in [0x000, STATES_ID, 0x7ff] {
                assert!(is_valid_frame(&gen_frame(id, &[0; 8])));
            }
        }

        #[test]
        fn accepts_empty_data() {
            assert!(is_valid_frame(&gen_frame(STATES_ID, &[])));
        }

        #[test]
        fn rejects_extended() {
            for id in [0x0000000, STATES_ID as u32, 0x1fffffff] {
                assert!(!is_valid_frame(&gen_extended(id, &[0; 8])));
            }
        }

        #[test]
        fn rejects_remote() {
            for len in 0..=8 {
                assert!(!is_valid_frame(&gen_remote(STATES_ID, len)));
            }
        }
    }

    #[embedded_test::tests]
    mod queue_msg {
        use crate::*;

        #[test]
        fn queues_frame() {
            assert!(queue_msg(0x0c0, &[1, 2, 3, 4, 5, 6, 7, 8]));
            check_tx(0x0c0, [1, 2, 3, 4, 5, 6, 7, 8]);
            check_tx_empty();
        }

        #[test]
        fn queues_empty_payload() {
            assert!(queue_msg(0x0c0, &[]));
            assert!(TX_QUEUE.try_receive().unwrap().data().is_empty());
            check_tx_empty();
        }

        #[test]
        fn keeps_order() {
            for id in [0x7ff, 0x000, 0x0c0] {
                assert!(queue_msg(id, &[0; 8]));
            }
            for id in [0x7ff, 0x000, 0x0c0] {
                check_tx(id, [0; 8]);
            }
            check_tx_empty();
        }

        #[test]
        fn rejects_bad_frame() {
            assert!(!queue_msg(0x0c0, &[0; 9]));
            assert!(!queue_msg(0x800, &[0; 8]));
            check_tx_empty();
        }

        #[test]
        fn drops_when_full() {
            fill_tx();
            assert!(!queue_msg(0x0c0, &[0xff; 8]));
            for _ in 0..TX_QUEUE.capacity() {
                check_tx(0x123, [0; 8]);
            }
            check_tx_empty();
        }

        #[test]
        fn recovers_after_full() {
            fill_tx();
            assert!(!queue_msg(0x0c0, &[0; 8]));
            check_tx(0x123, [0; 8]);
            assert!(queue_msg(0x0c0, &[0; 8]));
            assert!(!queue_msg(0x0c0, &[0; 8]));
        }
    }
}

mod motor_controller {
    #[embedded_test::tests]
    mod read {
        use crate::*;

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
        use crate::*;

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
        use crate::*;

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
        use crate::*;

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
}

mod pedals {
    #[embedded_test::tests]
    mod percent {
        use crate::*;
        use defmt::assert_eq;

        #[test]
        fn rejects_out_of_range() {
            for raw in [0, 49, 4001, 4095, u16::MAX] {
                assert_eq!(APPS1.percent(raw), None);
            }
        }

        #[test]
        fn accepts_range_limits() {
            assert_eq!(APPS1.percent(50), Some(0));
            assert_eq!(APPS1.percent(4000), Some(100));
        }

        #[test]
        fn reads_zero_in_deadband() {
            for raw in [149, 150, 151, 199, 200, 201] {
                assert_eq!(APPS1.percent(raw), Some(0));
            }
        }

        #[test]
        fn rounds_down() {
            assert_eq!(APPS1.percent(236), Some(0));
            assert_eq!(APPS1.percent(237), Some(1));
            assert_eq!(APPS1.percent(2049), Some(49));
            assert_eq!(APPS1.percent(2050), Some(50));
            assert_eq!(APPS1.percent(3899), Some(99));
        }

        #[test]
        fn caps_at_full() {
            for raw in [3900, 3901, 3999] {
                assert_eq!(APPS1.percent(raw), Some(100));
            }
        }

        #[test]
        fn never_decreases() {
            let mut last = 0;
            for raw in 50..=4000 {
                let percent = APPS1.percent(raw).unwrap();
                assert!(percent >= last && percent <= 100);
                last = percent;
            }
        }
    }

    #[embedded_test::tests]
    mod torque {
        use crate::*;
        use defmt::assert_eq;

        #[test]
        fn scales_percent() {
            assert_eq!(torque(0), 0);
            assert_eq!(torque(1), 10);
            assert_eq!(torque(50), 500);
            assert_eq!(torque(99), 990);
            assert_eq!(torque(100), 1000);
        }

        #[test]
        fn stays_in_limit_for_every_input() {
            for raw in 0..=u16::MAX {
                assert!(APPS1.percent(raw).map_or(0, torque) <= 1000);
            }
        }
    }
}

mod pcb {
    #[embedded_test::tests]
    mod clocks {
        use crate::*;

        #[test]
        fn accepts_config() {
            let peripherals = embassy_stm32::init(clocks::config());
            clocks::verify(&peripherals.RCC);
        }

        #[test]
        #[should_panic]
        fn rejects_missing_hse() {
            let peripherals = embassy_stm32::init(Default::default());
            clocks::verify(&peripherals.RCC);
        }

        #[test]
        #[should_panic]
        fn rejects_wrong_sys() {
            let mut config = clocks::config();
            config.rcc.sys = Sysclk::HSI;
            let peripherals = embassy_stm32::init(config);
            clocks::verify(&peripherals.RCC);
        }

        #[test]
        #[should_panic]
        fn rejects_wrong_can() {
            let mut config = clocks::config();
            config.rcc.mux.fdcansel = mux::Fdcansel::PCLK1;
            config.rcc.apb1_pre = APBPrescaler::DIV2;
            let peripherals = embassy_stm32::init(config);
            clocks::verify(&peripherals.RCC);
        }
    }

    #[embedded_test::tests]
    mod startup {
        use crate::*;

        #[test]
        fn check_reset_clears_flags() {
            let _peripherals = embassy_stm32::init(clocks::config());
            let csr = RCC.csr().read();
            assert!(csr.pinrstf() || csr.sftrstf());

            init::check_reset();
            block_for(Duration::from_millis(1));

            let csr = RCC.csr().read();
            assert!(!csr.pinrstf());
            assert!(!csr.borrstf());
            assert!(!csr.sftrstf());
            assert!(!csr.iwdgrstf());
            assert!(!csr.wwdgrstf());
            assert!(!csr.lpwrrstf());
        }

        #[test]
        async fn setup_queues_startup_msgs() {
            let spawner = unsafe { Spawner::for_current_executor() }.await;
            let _resources = init::setup(spawner);

            check_tx(PARAM_ID, [0x14, 0, 0x01, 0, 0, 0, 0, 0]);
            check_tx(PARAM_ID, [0x8e, 0, 0, 0, 0, 0, 0, 0]);
            check_tx(PARAM_ID, [0x8f, 0, 0, 0, 0, 0, 0, 0]);
            check_tx(PARAM_ID, [0x92, 0, 0, 0, 0, 0, 0, 0]);
            check_tx(PARAM_ID, [0xac, 0, 0, 0, 0, 0, 0, 0]);
            check_tx(CMD_ID, STOP);
            check_tx_empty();
            assert!(!MotorController.is_ready());
        }
    }

    #[embedded_test::tests]
    mod can_tasks {
        use crate::*;

        #[test]
        async fn skips_bus_error() {
            let _peripherals = embassy_stm32::init(clocks::config());
            let spawner = unsafe { Spawner::for_current_executor() }.await;
            spawner.spawn(can::tasks::input_processor(RX_QUEUE.dyn_receiver()).unwrap());

            assert!(RX_QUEUE.try_send(Err(BusError::Crc)).is_ok());
            let frame = gen_frame(STATES_ID, &[0; 8]);
            assert!(RX_QUEUE.try_send(Ok(frame)).is_ok());
            Timer::after_millis(10).await;

            assert!(RX_QUEUE.is_empty());
            assert!(MotorController.is_ready());
        }

        #[test]
        async fn drops_invalid_frames() {
            let _peripherals = embassy_stm32::init(clocks::config());
            let spawner = unsafe { Spawner::for_current_executor() }.await;
            spawner.spawn(can::tasks::input_processor(RX_QUEUE.dyn_receiver()).unwrap());

            let extended = gen_extended(STATES_ID as u32, &[0; 8]);
            let remote = gen_remote(STATES_ID, 8);
            assert!(RX_QUEUE.try_send(Ok(extended)).is_ok());
            assert!(RX_QUEUE.try_send(Ok(remote)).is_ok());
            Timer::after_millis(10).await;

            assert!(RX_QUEUE.is_empty());
            assert!(!MotorController.is_ready());
        }
    }

    #[embedded_test::tests]
    mod can_loopback {
        use crate::*;

        #[test]
        async fn frame_reaches_motor_controller() {
            let (can_tx, can_rx, can_props) = start_loopback().split();
            let spawner = unsafe { Spawner::for_current_executor() }.await;
            spawner.spawn(can::tasks::input_listener(can_rx, RX_QUEUE.dyn_sender()).unwrap());
            spawner.spawn(can::tasks::input_processor(RX_QUEUE.dyn_receiver()).unwrap());
            spawner.spawn(
                can::tasks::output_sender(can_tx, TX_QUEUE.dyn_receiver(), TX_QUEUE.dyn_sender())
                    .unwrap(),
            );

            assert!(!MotorController.is_ready());
            assert!(queue_msg(STATES_ID, &[0; 8]));
            Timer::after_millis(50).await;
            assert!(MotorController.is_ready());

            assert!(queue_msg(FAULTS_ID, &FAULT));
            Timer::after_millis(50).await;
            assert!(!MotorController.is_ready());

            assert!(queue_msg(FAULTS_ID, &[0; 8]));
            Timer::after_millis(50).await;
            assert!(MotorController.is_ready());

            can::log_bus_errs(&can_props);
            assert!(matches!(
                can_props.bus_error_mode(),
                BusErrorMode::ErrorActive
            ));
        }

        #[test]
        async fn filters_extended_and_remote() {
            let (can_tx, can_rx, _can_props) = start_loopback().split();
            let spawner = unsafe { Spawner::for_current_executor() }.await;
            spawner.spawn(can::tasks::input_listener(can_rx, RX_QUEUE.dyn_sender()).unwrap());
            spawner.spawn(
                can::tasks::output_sender(can_tx, TX_QUEUE.dyn_receiver(), TX_QUEUE.dyn_sender())
                    .unwrap(),
            );

            let extended = gen_extended(STATES_ID as u32, &[0; 8]);
            let remote = gen_remote(STATES_ID, 8);
            assert!(TX_QUEUE.try_send(extended).is_ok());
            assert!(TX_QUEUE.try_send(remote).is_ok());
            Timer::after_millis(50).await;
            assert!(TX_QUEUE.is_empty());
            assert!(RX_QUEUE.is_empty());

            assert!(queue_msg(STATES_ID, &[0; 8]));
            Timer::after_millis(50).await;
            assert!(matches!(RX_QUEUE.try_receive(), Ok(Ok(_))));
        }

        #[test]
        async fn drops_rx_when_full() {
            let (can_tx, can_rx, can_props) = start_loopback().split();
            let spawner = unsafe { Spawner::for_current_executor() }.await;
            spawner.spawn(can::tasks::input_listener(can_rx, RX_QUEUE.dyn_sender()).unwrap());
            spawner.spawn(
                can::tasks::output_sender(can_tx, TX_QUEUE.dyn_receiver(), TX_QUEUE.dyn_sender())
                    .unwrap(),
            );

            fill_tx();
            Timer::after_millis(50).await;
            assert!(TX_QUEUE.is_empty());
            assert!(RX_QUEUE.is_full());

            assert!(queue_msg(0x0c0, &[0; 8]));
            Timer::after_millis(50).await;
            for _ in 0..RX_QUEUE.capacity() {
                let frame = RX_QUEUE.try_receive().unwrap().unwrap();
                assert!(get_frame_id(&frame) == 0x123);
            }
            assert!(RX_QUEUE.is_empty());

            assert!(queue_msg(0x0c0, &[0; 8]));
            Timer::after_millis(50).await;
            assert!(RX_QUEUE.len() == 1);
            assert!(matches!(
                can_props.bus_error_mode(),
                BusErrorMode::ErrorActive
            ));
        }
    }
}
