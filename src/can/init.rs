use defmt::info;
use embassy_stm32::can::config::{FrameTransmissionConfig, NominalBitTiming};
use embassy_stm32::can::enums::TimingCalcError;
use embassy_stm32::can::filter::{StandardFilter as Filter, StandardFilterSlot as FilterSlot};
use embassy_stm32::can::util::calc_can_timings;
use embassy_stm32::can::{self, Can, CanConfigurator, OperatingMode};
use embassy_stm32::peripherals::{FDCAN1, PA11, PA12};
use embassy_stm32::{Peri, bind_interrupts};

use crate::clock::CAN_CLK;

// Bitrate/baudrate constant across car
pub const BITRATE: u32 = 500_000;

// Ensure bitrate matches clock
const _: () = assert!(BITRATE > 0 && CAN_CLK.0 % BITRATE == 0);

// Can interrupts routed into embassy
bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => can::IT0InterruptHandler<FDCAN1>;
    FDCAN1_IT1 => can::IT1InterruptHandler<FDCAN1>;
});

pub fn can_init<'d>(
    bus: Peri<'d, FDCAN1>,
    rx: Peri<'d, PA11>,
    tx: Peri<'d, PA12>,
    mode: OperatingMode,
) -> Result<Can<'d>, TimingCalcError> {
    let mut can = CanConfigurator::new(bus, rx, tx, Irqs);

    let krnl_clk = can.properties().kernel_input_clock();
    let timings = calc_can_timings(krnl_clk, BITRATE)?;
    let config = can
        .config()
        .set_nominal_bit_timing(NominalBitTiming {
            prescaler: timings.prescaler,
            seg1: timings.seg1,
            seg2: timings.seg2,
            sync_jump_width: timings.sync_jump_width,
        })
        .set_frame_transmit(FrameTransmissionConfig::ClassicCanOnly);
    can.set_config(config);

    // Route identifiers to Rx, FIFO
    can.properties()
        .set_standard_filter(FilterSlot::_0, Filter::accept_all_into_fifo0());

    let can = can.start(mode);
    info!(
        "CAN initialized:\n\
        \tBitrate: {} kbps,\n\
        \tMode: {},\n\
        \tKernerl clock: {} Hz,\n\
        \tPrescaler timing: {},\n\
        \tSEG1 timing: {},\n\
        \tSEG2 timing: {},\n\
        \tSync jump width: {}",
        BITRATE / 1_000,
        mode,
        krnl_clk.0,
        timings.prescaler.get(),
        timings.seg1.get(),
        timings.seg2.get(),
        timings.sync_jump_width.get()
    );

    Ok(can)
}
