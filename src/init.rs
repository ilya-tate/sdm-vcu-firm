use defmt::{error, info, println, unwrap};
use embassy_executor::Spawner;
use embassy_stm32::adc::{Adc, SampleTime};
use embassy_stm32::can::{OperatingMode, Properties};
use embassy_stm32::pac::RCC;
use embassy_stm32::peripherals::IWDG;
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::{Duration, Ticker};

use crate::can::devices::motor_controller::MotorController;
use crate::{can, clocks, pedals};

// TIMEOUT MUST OUTLAST EXPECTED LOOP ITERATION TIME
const WDG_TIMEOUT_US: u32 = 2_000_000;

#[cfg(debug_assertions)]
const CAN_MODE: OperatingMode = OperatingMode::InternalLoopbackMode;
#[cfg(not(debug_assertions))]
const CAN_MODE: OperatingMode = OperatingMode::NormalOperationMode;

pub struct Resources {
    pub wdg: IndependentWatchdog<'static, IWDG>,
    pub can_props: Properties,
    pub ticker: Ticker,
}

pub fn setup(spawner: Spawner) -> Resources {
    // Clock -> CAN timing calculated from can bus clock
    let mut peripherals = embassy_stm32::init(clocks::config());

    clocks::verify(&peripherals.RCC);

    check_reset();

    let can = match can::init::can_init(
        peripherals.FDCAN1,
        peripherals.PA11,
        peripherals.PA12,
        CAN_MODE,
    ) {
        Ok(can) => can,
        Err(e) => defmt::panic!("CAN Init Error: {}", e),
    };
    let mut adc = Adc::new(peripherals.ADC1, Default::default());

    let (can_tx, can_rx, can_props) = can.split();
    spawner.spawn(unwrap!(can::tasks::input_listener(
        can_rx,
        can::RX_QUEUE.dyn_sender(),
    )));
    spawner.spawn(unwrap!(can::tasks::input_processor(
        can::RX_QUEUE.dyn_receiver()
    )));
    spawner.spawn(unwrap!(can::tasks::output_sender(
        can_tx,
        can::TX_QUEUE.dyn_receiver(),
        can::TX_QUEUE.dyn_sender(),
    )));
    let _ = adc.blocking_read(&mut peripherals.PA0, SampleTime::CYCLES24_5);
    spawner.spawn(unwrap!(pedals::pedal_processor(pedals::Resources {
        adc,
        apps1: peripherals.PA0,
        apps2: peripherals.PA1,
        brake: peripherals.PB0,
    })));

    // Reset if loop stalls
    let mut wdg: IndependentWatchdog<'_, embassy_stm32::peripherals::IWDG> =
        IndependentWatchdog::new(peripherals.IWDG, WDG_TIMEOUT_US);
    wdg.unleash();

    MotorController.clear_faults();
    MotorController.read_config();
    MotorController.send_stop_cmd();

    println!("SDM VCU init done");

    let ticker = Ticker::every(Duration::from_millis(10));

    Resources {
        wdg,
        can_props,
        ticker,
    }
}

// Log wdg trigger reason, then clear flags
pub fn check_reset() {
    if RCC.csr().read().iwdgrstf() {
        error!(
            r"Down boy!
  .
 ..^____/
`-. ___ )
  ||  || mh"
        );
    }

    let csr = RCC.csr().read();
    info!(
        "Reset flags: pin={}\nbor={}\nstf={}\niwdg={}\nwwdg={}\nlpwr={}",
        csr.pinrstf(),
        csr.borrstf(),
        csr.sftrstf(),
        csr.iwdgrstf(),
        csr.wwdgrstf(),
        csr.lpwrrstf()
    );

    // Clear reset flags
    RCC.csr().modify(|w| w.set_rmvf(true));
}
