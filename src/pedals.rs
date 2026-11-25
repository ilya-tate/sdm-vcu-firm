use core::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use embassy_stm32::Peri;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::peripherals::{ADC1, PA0, PA1, PB0};
use embassy_time::{Duration, Instant, Ticker};

// Brake checking
const BRAKE_THRESH: u8 = 10;
const BRAKE_PLAUS_THRESH: u8 = 25;
const BRAKE_PLAUS_RESET_THRESH: u8 = 5;

// APPS checking
const APPS_DIFF_THRESH: u8 = 10;
const APPS_FAULT_TIME: Duration = Duration::from_millis(100);

// Sensor
const MAX_RANGE: u16 = 100;
const MAX_TORQUE: u16 = 1000;

// Task loop
const TICK: Duration = Duration::from_millis(10);

pub static TORQUE: AtomicU16 = AtomicU16::new(0);

pub struct Resources {
    pub adc: Adc<'static, ADC1>,
    pub apps1: Peri<'static, PA0>,
    pub apps2: Peri<'static, PA1>,
    pub brake: Peri<'static, PB0>,
}

// TOOO:
// Calibrate test values for PedalSensor instances

pub struct PedalSensor {
    min: u16,
    max: u16,
    deadzone: u16,
    is_released: AtomicBool,
}

pub static APPS1: PedalSensor = PedalSensor {
    min: 150,
    max: 3900,
    deadzone: 50,
    is_released: AtomicBool::new(false),
};

pub static APPS2: PedalSensor = PedalSensor {
    min: 150,
    max: 3900,
    deadzone: 50,
    is_released: AtomicBool::new(false),
};

pub static BRAKE: PedalSensor = PedalSensor {
    min: 150,
    max: 3900,
    deadzone: 50,
    is_released: AtomicBool::new(false),
};

impl PedalSensor {
    pub const fn percent(&self, raw: u16) -> Option<u8> {
        // Out of sensor range
        if raw < self.min.saturating_sub(MAX_RANGE) || raw > self.max.saturating_add(MAX_RANGE) {
            return None;
        }

        // Within deadzone
        let start = self.min + self.deadzone;
        if raw <= start {
            return Some(0);
        }

        // Fully pressed
        if raw >= self.max {
            return Some(100);
        }

        // Read whatever input
        Some(((raw - start) as u32 * 100 / (self.max - start) as u32) as u8)
    }

    pub fn is_released(&self) -> bool {
        self.is_released.load(Ordering::Relaxed)
    }
}

pub const fn torque(percent: u8) -> u16 {
    (percent as u32 * MAX_TORQUE as u32 / 100) as u16
}

pub fn check_apps_plaus(apps1: u8, apps2: u8, fault_at: &mut Option<Instant>) -> bool {
    // Ensure both APPS sensors are reading similarly
    if apps1.abs_diff(apps2) <= APPS_DIFF_THRESH {
        *fault_at = None;
        return true;
    }

    fault_at.get_or_insert_with(Instant::now).elapsed() < APPS_FAULT_TIME
}

pub const fn check_brake_fault(is_fault: bool, throttle: u8, brake: u8) -> bool {
    if brake >= BRAKE_THRESH && throttle > BRAKE_PLAUS_THRESH {
        return true;
    } else if throttle < BRAKE_PLAUS_RESET_THRESH {
        return false;
    }

    return is_fault;
}

#[embassy_executor::task]
pub async fn pedal_processor(mut resources: Resources) -> ! {
    let mut ticker = Ticker::every(TICK);
    let mut is_brake_fault = true;
    let mut prev_apps_fault_at: Option<Instant> = None;

    loop {
        ticker.next().await;

        let apps1_read = resources
            .adc
            .blocking_read(&mut resources.apps1, SampleTime::CYCLES24_5);
        let apps2_read = resources
            .adc
            .blocking_read(&mut resources.apps2, SampleTime::CYCLES24_5);
        let brake_read = resources
            .adc
            .blocking_read(&mut resources.brake, SampleTime::CYCLES24_5);

        let apps1 = APPS1.percent(apps1_read);
        let apps2 = APPS2.percent(apps2_read);
        let brake = BRAKE.percent(brake_read);

        APPS1.is_released.store(apps1 == Some(0), Ordering::Relaxed);
        APPS2.is_released.store(apps2 == Some(0), Ordering::Relaxed);
        BRAKE.is_released.store(brake == Some(0), Ordering::Relaxed);

        let throttle = match (apps1, apps2, brake) {
            (Some(apps1), Some(apps2), Some(brake)) => {
                is_brake_fault = check_brake_fault(is_brake_fault, apps1.max(apps2), brake);
                let is_apps_plaus = check_apps_plaus(apps1, apps2, &mut prev_apps_fault_at);
                let is_send_torque_allowed = is_apps_plaus && !is_brake_fault;
                if is_send_torque_allowed {
                    apps1.min(apps2)
                } else {
                    0
                }
            }

            _ => 0,
        };

        TORQUE.store(torque(throttle), Ordering::Relaxed);
    }
}
