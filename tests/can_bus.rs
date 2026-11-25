use defmt::assert_eq;
use embassy_stm32::can::{Can, Frame, OperatingMode};
use embedded_can::StandardId;

use sdm_vcu_firm::can::{self, TX_QUEUE, get_frame_id, is_valid_frame, queue_msg, try_frame_build};
use sdm_vcu_firm::clocks;

use crate::motor_controller::STATES_ID;

pub fn gen_frame(id: u16, data: &[u8]) -> Frame {
    Frame::new_standard(id, data).unwrap()
}

pub fn gen_extended(id: u32, data: &[u8]) -> Frame {
    Frame::new_extended(id, data).unwrap()
}

pub fn gen_remote(id: u16, len: usize) -> Frame {
    Frame::new_remote(StandardId::new(id).unwrap(), len).unwrap()
}

pub fn check_tx(id: u16, data: [u8; 8]) {
    let frame = TX_QUEUE.try_receive().unwrap();
    assert_eq!(get_frame_id(&frame), id as u32);
    assert_eq!(frame.data(), &data[..]);
}

pub fn check_tx_empty() {
    assert!(TX_QUEUE.try_receive().is_err());
}

pub fn fill_tx() {
    for _ in 0..TX_QUEUE.capacity() {
        assert!(queue_msg(0x123, &[0; 8]));
    }
}

pub fn start_loopback() -> Can<'static> {
    let peripherals = embassy_stm32::init(clocks::config());
    can::init::can_init(
        peripherals.FDCAN1,
        peripherals.PA11,
        peripherals.PA12,
        OperatingMode::InternalLoopbackMode,
    )
    .unwrap()
}

#[embedded_test::tests]
mod get_frame_id {
    use super::*;
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
    use super::*;
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
    use super::*;

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
    use super::*;

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
