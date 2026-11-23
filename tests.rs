#![no_std]
#![no_main]

#[embedded_test::tests]
mod tests {
    use defmt_rtt as _;
    use sdm_vcu_firm as _;

    #[test]
    fn it_runs() {
        assert!(true);
    }
}
