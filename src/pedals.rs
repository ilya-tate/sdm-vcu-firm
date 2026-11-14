const MAX_RANGE: u16 = 100;
const MAX_TORQUE: u16 = 1000;

#[derive(Clone, Copy)]
pub struct PedalSensor {
    min: u16,
    max: u16,
    deadband: u16,
}

pub const APPS1: PedalSensor = PedalSensor {
    // Currently test values
    // Needs to be calibrated
    min: 150,
    max: 3900,
    deadband: 50,
};

impl PedalSensor {
    pub const fn percent(&self, raw: u16) -> Option<u8> {
        if raw < self.min.saturating_sub(MAX_RANGE) || raw > self.max.saturating_add(MAX_RANGE) {
            return None;
        }

        let start = self.min + self.deadband;
        if raw <= start {
            return Some(0);
        }
        if raw >= self.max {
            return Some(100);
        }

        Some(((raw - start) as u32 * 100 / (self.max - start) as u32) as u8)
    }
}

pub const fn torque(percent: u8) -> u16 {
    (percent as u32 * MAX_TORQUE as u32 / 100) as u16
}
