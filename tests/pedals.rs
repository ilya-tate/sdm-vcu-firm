use sdm_vcu_firm::pedals::{APPS1, APPS2, BRAKE, check_brake_fault, torque};

#[embedded_test::tests]
mod percent {
    use super::*;
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
        assert_eq!(APPS2.percent(2050), Some(50));
        assert_eq!(BRAKE.percent(2050), Some(50));
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
    use super::*;
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

#[embedded_test::tests]
mod brake_fault {
    use super::*;

    #[test]
    fn sets_on_brake_and_throttle() {
        assert!(check_brake_fault(false, 26, 10));
    }

    #[test]
    fn needs_throttle_over_thresh() {
        assert!(!check_brake_fault(false, 25, 10));
    }

    #[test]
    fn needs_brake_at_thresh() {
        assert!(!check_brake_fault(false, 26, 9));
    }

    #[test]
    fn holds_until_throttle_released() {
        assert!(check_brake_fault(true, 5, 0));
        assert!(check_brake_fault(true, 30, 0));
    }

    #[test]
    fn resets_under_reset_thresh() {
        assert!(!check_brake_fault(true, 4, 0));
        assert!(!check_brake_fault(true, 4, 100));
    }
}
