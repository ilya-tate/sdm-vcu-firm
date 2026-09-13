use esp_hal::gpio::{Level, InputPin, Output, OutputPin, OutputConfig};
use esp_hal::twai::{BaudRate, TwaiRx, TwaiTx, TwaiConfiguration, TwaiMode};

pub fn tranceiver_enable(standby_pin: impl OutputPin + 'static) -> Output<'static> {
    Output::new(
        standby_pin,
        Level::Low, // Set polarity low
        OutputConfig::default()
    )
}


// I thought about using async, but am hesitant (bottlenecks)
// Blocking could result in 'packet' loss (yes it says blocking, unrelated)
// Im just gonna do a check in bin/main.rs,
// it seems like the best of both worlds
pub fn can_init(
    twai0:      esp_hal::peripherals::TWAI0,
    pin_rx:     impl InputPin,
    pin_tx:     impl OutputPin,
    baud_rate:  BaudRate
) -> (TwaiTx<'static, esp_hal::Blocking>, TwaiRx<'static, esp_hal::Blocking>) {
    TwaiConfiguration::new(twai0, pin_rx, pin_tx, baud_rate, TwaiMode::Normal)
        .start()
        .split()
}

