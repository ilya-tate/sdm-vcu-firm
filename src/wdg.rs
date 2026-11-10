use defmt::error;
use embassy_stm32::pac::RCC;

/// Check if the last reset was caused by the independent watchdog (IWDG) and log an error message if so. Clear the reset flags afterward.
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

    // Clear reset flags
    RCC.csr().modify(|w| w.set_rmvf(true));
}
