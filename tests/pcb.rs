use embassy_executor::Spawner;
use embassy_stm32::can::enums::{BusError, BusErrorMode};
use embassy_stm32::pac::RCC;
use embassy_stm32::rcc::{APBPrescaler, Sysclk, mux};
use embassy_time::{Duration, Timer, block_for};

use sdm_vcu_firm::can::devices::motor_controller::MotorController;
use sdm_vcu_firm::can::{self, RX_QUEUE, TX_QUEUE, get_frame_id, queue_msg};
use sdm_vcu_firm::init;
use sdm_vcu_firm::pedals::check_apps_plaus;

use crate::can_bus::{
    check_tx, check_tx_empty, fill_tx, gen_extended, gen_frame, gen_remote, start_loopback,
};
use crate::motor_controller::{CMD_ID, FAULT, FAULTS_ID, PARAM_ID, STATES_ID, STOP};

#[embedded_test::tests]
mod clocks {
    use super::*;
    use sdm_vcu_firm::clocks;

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
    use super::*;
    use sdm_vcu_firm::clocks;

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
    use super::*;
    use sdm_vcu_firm::clocks;

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
    use super::*;

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

#[embedded_test::tests]
mod apps_plaus {
    use super::*;
    use sdm_vcu_firm::clocks;

    #[test]
    fn accepts_matching_sensors() {
        let _peripherals = embassy_stm32::init(clocks::config());
        let mut fault_at = None;
        assert!(check_apps_plaus(40, 50, &mut fault_at));
        assert!(fault_at.is_none());
    }

    #[test]
    fn starts_timer_on_mismatch() {
        let _peripherals = embassy_stm32::init(clocks::config());
        let mut fault_at = None;
        assert!(check_apps_plaus(40, 51, &mut fault_at));
        assert!(fault_at.is_some());
    }

    #[test]
    fn allows_mismatch_under_fault_time() {
        let _peripherals = embassy_stm32::init(clocks::config());
        let mut fault_at = None;
        check_apps_plaus(0, 50, &mut fault_at);
        block_for(Duration::from_millis(50));
        assert!(check_apps_plaus(0, 50, &mut fault_at));
    }

    #[test]
    fn rejects_mismatch_past_fault_time() {
        let _peripherals = embassy_stm32::init(clocks::config());
        let mut fault_at = None;
        check_apps_plaus(0, 50, &mut fault_at);
        block_for(Duration::from_millis(110));
        assert!(!check_apps_plaus(0, 50, &mut fault_at));
    }

    #[test]
    fn recovers_and_resets_timer() {
        let _peripherals = embassy_stm32::init(clocks::config());
        let mut fault_at = None;
        check_apps_plaus(0, 50, &mut fault_at);
        block_for(Duration::from_millis(110));
        assert!(!check_apps_plaus(0, 50, &mut fault_at));
        assert!(check_apps_plaus(50, 50, &mut fault_at));
        assert!(fault_at.is_none());
        assert!(check_apps_plaus(0, 50, &mut fault_at));
    }
}
