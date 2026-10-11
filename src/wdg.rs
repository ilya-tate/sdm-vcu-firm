use defmt::error;
use embassy_stm32::pac::RCC;

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
