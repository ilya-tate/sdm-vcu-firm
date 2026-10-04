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

    use rtic_monotonics::Monotonic;
    use rtic_monotonics::systick::{Systick, *};

    /// There are no resources shared across tasks.
    #[shared]
    struct Shared {}

    /// These resources are local to individual tasks.
    #[local]
    struct Local {
        led: board::Led, // Pin 13
        poller: logging::Poller,
        can: Result<can::CanBus, can::init::InitErr>, // Pins 0, 1
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

        let can = can::CanBus::new(&mut ccm, pins.p0, pins.p1);

        Systick::start(
            cx.core.SYST,
            board::ARM_FREQUENCY,
            rtic_monotonics::create_systick_token!(),
        );

        blink::spawn().unwrap();
        can_test::spawn().unwrap();
        (Shared {}, Local { led, poller, can })
    }

    #[task(local = [led])]
    async fn blink(cx: blink::Context) {
        loop {
            cx.local.led.toggle();
            Systick::delay(500.millis()).await;
        }
    }

    // Bench test: one standard data frame per second, RX polling every 1 ms.
    #[task(local = [can])]
    async fn can_test(cx: can_test::Context) {
        let frame = can::frame::Frame::new(0x123, &[1, 2, 3, 4]).unwrap();
        let mut next_send = Systick::now() + 2.secs();
        let mut pending = false;
        loop {
            match cx.local.can.as_mut() {
                Ok(bus) => {
                    if bus.take_tx_complete() {
                        pending = false;
                        log::info!("CAN TX complete id=123 data=01 02 03 04");
                    }
                    // Bound each drain so a busy bus cannot starve other tasks.
                    for _ in 0..14 {
                        match bus.try_receive() {
                            Ok(Some(received)) => {
                                log::info!(
                                    "CAN RX id={:03X} data={:02X?} overrun={}",
                                    received.frame.id(),
                                    received.frame.payload(),
                                    received.overrun
                                );
                            }
                            Ok(None) => break,
                            Err(e) => log::warn!("CAN RX error: {:?}", e),
                        }
                    }
                    if Systick::now() >= next_send {
                        if pending {
                            log::warn!("CAN TX still pending: check peer, bitrate and wiring");
                        } else {
                            match bus.try_transmit(&frame) {
                                Ok(()) => {
                                    pending = true;
                                    log::info!(
                                        "CAN TX queued id=123 @ {} kbps",
                                        can::BITRATE / 1000
                                    );
                                }
                                Err(e) => log::warn!("CAN TX error: {:?}", e),
                            }
                        }
                        let (esr, ecr) = bus.error_status();
                        log::info!(
                            "CAN ESR1={:08X} TX errors={} RX errors={}",
                            esr,
                            ecr & 0xff,
                            (ecr >> 8) & 0xff
                        );
                        next_send = Systick::now() + 1.secs();
                    }
                }
                Err(e) => {
                    // Repeat so an early USB startup message cannot hide failure.
                    log::error!("CAN2 init failure: {:?}", e);
                    Systick::delay(1.secs()).await;
                }
            }
            Systick::delay(1.millis()).await;
        }
    }

    #[task(binds = USB_OTG1, local = [poller])]
    fn log_over_usb(cx: log_over_usb::Context) {
        cx.local.poller.poll();
    }
}
