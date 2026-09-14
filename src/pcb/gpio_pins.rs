// TODO: Check pins names :P

use esp_idf_hal::gpio::{Gpio1, Gpio2, Gpio3, Gpio4, Gpio5, Gpio6, Gpio7, Gpio8, Pins};

// Constructor stuff
pub struct GpioPins {
    // CAN3
    pub     can3_rx:        Gpio1,          // i
    pub     can3_tx:        Gpio2,          // o
    pub     can3_stb:       Gpio3,          // standby
    pub     can3_term:      Gpio7,          // termination

    // Can4
    pub     can4_rx:        Gpio4,          // i
    pub     can4_tx:        Gpio5,          // o
    pub     can4_stb:       Gpio6,          // standby
    pub     can4_term:      Gpio8,          // termination

    // TODO: Can 1 and 2
}

impl GpioPins {
    pub fn pins_map(pins: Pins) -> Self {
        Self {
            // CAN3
            can3_rx:        pins.gpio1,     // i
            can3_tx:        pins.gpio2,     // o
            can3_stb:       pins.gpio3,     // standby
            can3_term:      pins.gpio7,     // termination
                                        
            // CAN4
            can4_rx:        pins.gpio4,     // i
            can4_tx:        pins.gpio5,     // o
            can4_stb:       pins.gpio6,     // standby
            can4_term:      pins.gpio8      // termination
        }
    }
}



