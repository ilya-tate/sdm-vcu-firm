#![no_std]
#![no_main]

mod can;

use teensy4_panic as _;

#[rtic::app(device = teensy4_bsp, peripherals = true, dispatchers = [KPP])]
mod app {
    use bsp::board;
    use teensy4_bsp as bsp;

    use crate::can;

    use imxrt_log as logging;

    use board::t41;

    use rtic_monotonics::systick::{Systick, *};

    /// There are no resources shared across tasks.
    #[shared]
    struct Shared {}

    /// These resources are local to individual tasks.
    #[local]
    struct Local {
        led: board::Led, // Pin 13
        poller: logging::Poller,
        can: Option<can::CanBus>, // Pins 0, 1
    }

    #[init]
    fn init(cx: init::Context) -> (Shared, Local) {
        let board::Resources {
            mut gpio2,
            pins,
            usb,
            mut ccm,
            ..
        } = t41(cx.device);

        let led = board::led(&mut gpio2, pins.p13);
        let poller = logging::log::usbd(usb, logging::Interrupts::Enabled).unwrap();

        let can = match can::CanBus::new(&mut ccm, pins.p0, pins.p1) {
            Ok(bus) => {
                log::info!("CAN2 @ {} kbps", can::BITRATE / 1000);
                Some(bus)
            }

            Err(e) => {
                log::error!("CAN2 init failure: {:?}", e);
                None
            }
        };

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        blink::spawn().unwrap();
        (Shared {}, Local { led, poller, can })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        let mut count = 0u32;
        loop {
            cx.local.led.toggle();
            Systick::delay(500.millis()).await;

            log::info!("Hello from your Teensy 4! The count is {count}");
            if count % 7 == 0 {
                log::warn!("Here's a warning at count {count}");
            }
            if count % 23 == 0 {
                log::error!("Here's an error at count {count}");
            }

            count = count.wrapping_add(1);
        }
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
