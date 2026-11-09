use defmt::{error, info};
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
