use embassy_stm32::pac::RCC;
use embassy_stm32::peripherals::RCC as RccPeri;
use embassy_stm32::rcc::{self, Hse, HseMode, Sysclk, mux};
use embassy_stm32::time::{Hertz, MaybeHertz, mhz};
use embassy_stm32::{Config, Peri};

// Clocks set for NUCLEO-G474RE
// Forced to be equal without PLLs
pub const HSE_CLK: Hertz = mhz(24); // Crystal x3 @ 24mhz
pub const SYS_CLK: Hertz = HSE_CLK; // Core/bus clock
pub const CAN_CLK: Hertz = HSE_CLK; // canFD bus, divisible by bitrate
// Clock limits
const HSE_CLK_MIN: Hertz = mhz(4);
const HSE_CLK_MAX: Hertz = mhz(48);
const SYS_CLK_MAX: Hertz = mhz(170);
const CAN_CLK_MAX: Hertz = mhz(80);

/*
* Ensures follow max clock speeds,
* "const _" sets at compile rather than runtime.
* ".0" compares them as u32 instead of as a Hertz.
* This all happens at compile => slower build, but faster runtime.
*/
const _: () = assert!(HSE_CLK.0 >= HSE_CLK_MIN.0);
const _: () = assert!(HSE_CLK.0 <= HSE_CLK_MAX.0);
const _: () = assert!(SYS_CLK.0 <= SYS_CLK_MAX.0);
const _: () = assert!(CAN_CLK.0 <= CAN_CLK_MAX.0);

pub fn config() -> Config {
    let mut config = Config::default();

    // Enable HSE and set clock speeds
    config.rcc.hse = Some(Hse {
        freq: HSE_CLK,
        mode: HseMode::Oscillator,
    });
    config.rcc.sys = Sysclk::HSE;
    config.rcc.mux.fdcansel = mux::Fdcansel::HSE;
    config.rcc.mux.adc12sel = mux::Adcsel::SYS;

    config
}

// Ensures configged clocks equal board clocks
pub fn verify(rcc_peri: &Peri<'_, RccPeri>) {
    let clocks = rcc::clocks(rcc_peri);

    let can = match RCC.ccipr().read().fdcansel() {
        mux::Fdcansel::HSE => clocks.hse,
        mux::Fdcansel::PLL1_Q => clocks.pll1_q,
        _ => clocks.pclk1,
    };

    check("HSE", HSE_CLK, clocks.hse);
    check("SYS", SYS_CLK, clocks.sys);
    check("CAN", CAN_CLK, can);
}

fn check(name: &str, expected: Hertz, actual: MaybeHertz) {
    let actual = actual.to_hertz().map_or(0, |hz| hz.0);
    let diff = actual as i64 - expected.0 as i64;

    defmt::assert!(
        diff == 0,
        "Clock Error: {} is {} Hz, expected {} Hz, off by {} Hz",
        name,
        actual,
        expected.0,
        diff
    );
}
