#[cfg(feature = "atsam4e")]
use crate::pac::{pmc, EFC, PMC};

#[cfg(feature = "atsam4s")]
use crate::pac::{pmc, EFC0, EFC1, PMC};

use crate::time::Hertz;
use crate::BorrowUnchecked;

use core::marker::PhantomData;
use cortex_m::interrupt;

lazy_static! {
    static ref MASTER_CLOCK_FREQUENCY: Hertz = calculate_master_clock_frequency_static();
}

// called by pre_init()
#[cfg(feature = "atsam4e")]
pub fn init(pmc: &mut PMC, efc: &mut EFC) {
    set_flash_wait_states_to_maximum(efc);
    let master_clock_frequency = setup_main_clock(pmc);
    set_flash_wait_states_to_match_frequence(efc, master_clock_frequency);
}

// called by pre_init()
#[cfg(feature = "atsam4s")]
pub fn init(pmc: &mut PMC, efc0: &mut EFC0, efc1: &mut EFC1) {
    set_flash_wait_states_to_maximum(efc0, efc1);
    let master_clock_frequency = setup_main_clock(pmc);
    set_flash_wait_states_to_match_frequence(efc0, efc1, master_clock_frequency);
}

pub fn get_master_clock_frequency() -> Hertz {
    *MASTER_CLOCK_FREQUENCY
}

fn setup_main_clock(pmc: &mut PMC) -> Hertz {
    switch_main_clock_to_fast_rc_12mhz(pmc);

    wait_for_main_clock_ready(&pmc);

    // Set up the PLL for 120Mhz operation (12Mhz RC * (10 / 1) = 120Mhz)
    let multiplier: u16 = 10;
    let divider: u8 = 1;
    enable_plla_clock(pmc, multiplier, divider);
    wait_for_plla_lock(&pmc);

    let prescaler = 0; // 0 = no prescaling.
    switch_master_clock_to_plla(pmc, prescaler);

    calculate_master_clock_frequency(&pmc)
}

fn calculate_master_clock_frequency(pmc: &PMC) -> Hertz {
    let mut mclk_freq = match pmc.pmc_mckr.read().css().bits() {
        0 => {
            // Slow clock
            panic!("Unsupported clock source: Slow clock.")
        }
        1 => {
            // Main clock
            panic!("Unsupported clock source: Main clock.")
        }
        2 => {
            // PLL
            let mut mclk_freq: u32 = match pmc.ckgr_mor.read().moscsel().bit_is_set() {
                true => 12000000,
                false => match pmc.ckgr_mor.read().moscrcf().bits() {
                    0 => 4000000,
                    1 => 8000000,
                    2 => 12000000,
                    _ => panic!("Unexpected value detected ready from pmc.ckgr_mor.moscrcf"),
                },
            };

            let plla_clock_source: u8 = 2; // 2 = PLLA
            if pmc.pmc_mckr.read().css().bits() == plla_clock_source {
                mclk_freq *= (pmc.ckgr_pllar.read().mula().bits() + 1) as u32;
                mclk_freq /= (pmc.ckgr_pllar.read().diva().bits()) as u32;
            }

            mclk_freq
        }
        _ => panic!("Invalid value found in PMC_MCKR.CSS"),
    };

    // Factor in the prescaler
    mclk_freq = match pmc.pmc_mckr.read().pres().bits() {
        7 => mclk_freq / 3, // Special case for a 3 prescaler
        prescaler => mclk_freq >> prescaler,
    };

    Hertz(mclk_freq)
}

fn calculate_master_clock_frequency_static() -> Hertz {
    interrupt::free(|_| PMC::borrow_unchecked(|pmc| calculate_master_clock_frequency(&pmc)))
}

fn get_flash_wait_states_for_clock_frequency(clock_frequency: Hertz) -> u8 {
    match clock_frequency {
        c if c.0 < 20000000 => 0,
        c if c.0 < 40000000 => 1,
        c if c.0 < 60000000 => 2,
        c if c.0 < 80000000 => 3,
        c if c.0 < 100000000 => 4,
        c if c.0 < 123000000 => 5,
        _ => panic!(
            "Invalid frequency provided to get_flash_wait_states(): {} ",
            clock_frequency.0
        ),
    }
}

#[cfg(feature = "atsam4e")]
fn set_flash_wait_states_to_maximum(efc: &mut EFC) {
    efc.fmr
        .modify(|_, w| unsafe { w.fws().bits(5).cloe().set_bit() });
}

#[cfg(feature = "atsam4s")]
fn set_flash_wait_states_to_maximum(efc0: &mut EFC0, efc1: &mut EFC1) {
    efc0.fmr
        .modify(|_, w| unsafe { w.fws().bits(5).cloe().set_bit() });
    efc1.fmr
        .modify(|_, w| unsafe { w.fws().bits(5).cloe().set_bit() });
}

#[cfg(feature = "atsam4e")]
fn set_flash_wait_states_to_match_frequence(efc: &mut EFC, clock_frequency: Hertz) {
    let wait_state_count = get_flash_wait_states_for_clock_frequency(clock_frequency);

    efc.fmr
        .modify(|_, w| unsafe { w.fws().bits(wait_state_count).cloe().set_bit() });
}

#[cfg(feature = "atsam4s")]
fn set_flash_wait_states_to_match_frequence(
    efc0: &mut EFC0,
    efc1: &mut EFC1,
    clock_frequency: Hertz,
) {
    let wait_state_count = get_flash_wait_states_for_clock_frequency(clock_frequency);

    efc0.fmr
        .modify(|_, w| unsafe { w.fws().bits(wait_state_count).cloe().set_bit() });
    efc1.fmr
        .modify(|_, w| unsafe { w.fws().bits(wait_state_count).cloe().set_bit() });
}

fn switch_main_clock_to_fast_rc_12mhz(pmc: &mut PMC) {
    enable_fast_rc_oscillator(pmc);
    wait_for_fast_rc_oscillator_to_stabilize(pmc);
    change_fast_rc_oscillator_to_12_mhz(pmc);
    wait_for_fast_rc_oscillator_to_stabilize(pmc);
    switch_to_fast_rc_oscillator(pmc);
}

fn enable_fast_rc_oscillator(pmc: &mut PMC) {
    pmc.ckgr_mor
        .modify(|_, w| unsafe { w.key().bits(0x37).moscrcen().set_bit() });
}

fn change_fast_rc_oscillator_to_12_mhz(pmc: &mut PMC) {
    pmc.ckgr_mor
        .modify(|_, w| unsafe { w.key().bits(0x37).moscrcf()._12_mhz() });
}

fn switch_to_fast_rc_oscillator(pmc: &mut PMC) {
    pmc.ckgr_mor
        .modify(|_, w| unsafe { w.key().bits(0x37).moscsel().clear_bit() });
}

fn wait_for_fast_rc_oscillator_to_stabilize(pmc: &PMC) {
    while pmc.pmc_sr.read().moscrcs().bit_is_clear() {}
}

fn is_main_clock_ready(pmc: &PMC) -> bool {
    pmc.pmc_sr.read().moscsels().bit_is_set()
}

fn wait_for_main_clock_ready(pmc: &PMC) {
    while !is_main_clock_ready(pmc) {}
}

fn enable_plla_clock(pmc: &mut PMC, multiplier: u16, divider: u8) {
    disable_plla_clock(pmc);

    // NOTE: the datasheet indicates the multplier used it MULA + 1 - hence the subtraction when setting the multiplier.
    pmc.ckgr_pllar.modify(|_, w| unsafe {
        w.one()
            .set_bit()
            .pllacount()
            .bits(0x3f)
            .mula()
            .bits(multiplier - 1)
            .diva()
            .bits(divider)
    });
}

fn disable_plla_clock(pmc: &mut PMC) {
    pmc.ckgr_pllar
        .modify(|_, w| unsafe { w.one().set_bit().mula().bits(0) });
}

fn is_plla_locked(pmc: &PMC) -> bool {
    pmc.pmc_sr.read().locka().bit_is_set()
}

fn wait_for_plla_lock(pmc: &PMC) {
    while !is_plla_locked(pmc) {}
}

fn switch_master_clock_to_plla(pmc: &mut PMC, prescaler: u8) {
    // Set the master clock prescaler
    pmc.pmc_mckr.modify(|_, w| w.pres().bits(prescaler));

    wait_for_master_clock_ready(pmc);

    // Set the master clock source to PLLA
    // BUGBUG: What requires the 'unsafe' on SAM4?  SVD issue?
    let clock_source: u8 = 2; // 2 = PLLA
    #[cfg(feature = "atsam4e")]
    pmc.pmc_mckr
        .modify(|_, w| unsafe { w.css().bits(clock_source) });

    #[cfg(feature = "atsam4s")]
    pmc.pmc_mckr.modify(|_, w| w.css().bits(clock_source));

    wait_for_master_clock_ready(pmc);
}

fn is_master_clock_ready(pmc: &PMC) -> bool {
    pmc.pmc_sr.read().mckrdy().bit_is_set()
}

fn wait_for_master_clock_ready(pmc: &PMC) {
    while !is_master_clock_ready(pmc) {}
}

// Peripheral Clock State
#[derive(Default)]
pub struct Enabled;

#[derive(Default)]
pub struct Disabled;

#[derive(Default)]
pub struct PeripheralClock<STATE> {
    _state: PhantomData<STATE>,
}

macro_rules! peripheral_clocks {
    (
        $($PeripheralType:ident, $peripheral_ident:ident, $i:expr,)+
    ) => {
        #[derive(Default)]
        pub struct PeripheralClocks {
            $(
                pub $peripheral_ident: $PeripheralType<Disabled>,
            )+
        }

        impl PeripheralClocks {
            pub fn new() -> Self {
                PeripheralClocks {
                    $(
                        $peripheral_ident: $PeripheralType { _state: PhantomData },
                    )+
                }
            }
        }

        $(
            #[derive(Default)]
            pub struct $PeripheralType<STATE> {
                _state: PhantomData<STATE>,
            }

            impl<STATE> $PeripheralType<STATE> {
                pub(crate) fn pcer0(&mut self) -> &pmc::PMC_PCER0 {
                    unsafe { &(*PMC::ptr()).pmc_pcer0 }
                }

                pub(crate) fn pcer1(&mut self) -> &pmc::PMC_PCER1 {
                    unsafe { &(*PMC::ptr()).pmc_pcer1 }
                }

                pub(crate) fn pcdr0(&mut self) -> &pmc::PMC_PCDR0 {
                    unsafe { &(*PMC::ptr()).pmc_pcdr0 }
                }

                pub fn into_enabled_clock(mut self) -> $PeripheralType<Enabled> {
                    if $i <= 31 {
                        let shift = $i;
                        self.pcer0().write_with_zero(|w| unsafe { w.bits(1 << shift) });
                    }
                    else {
                        let shift = ($i - 32);
                        self.pcer1().write_with_zero(|w| unsafe { w.bits(1 << shift) });
                    }
                    $PeripheralType { _state: PhantomData }
                }

                pub fn into_disabled_clock(mut self) -> $PeripheralType<Disabled> {
                    if $i <= 31 {
                        let shift = $i;
                        self.pcdr0().write_with_zero(|w| unsafe { w.bits(1 << shift) });
                    }
                    else {
                        let shift = ($i - 32);
                        self.pcdr0().write_with_zero(|w| unsafe { w.bits(1 << shift) });
                    }
                    $PeripheralType { _state: PhantomData }
                }

                pub fn frequency(&self) -> Hertz {
                    get_master_clock_frequency()
                }
            }
        )+
    }
}

#[cfg(feature = "atsam4e")]
peripheral_clocks!(
    UART0Clock,
    uart_0,
    7,
    StaticMemoryControllerClock,
    static_memory_controller,
    8,
    ParallelIOControllerAClock,
    parallel_io_controller_a,
    9,
    ParallelIOControllerBClock,
    parallel_io_controller_b,
    10,
    ParallelIOControllerCClock,
    parallel_io_controller_c,
    11,
    ParallelIOControllerDClock,
    parallel_io_controller_d,
    12,
    ParallelIOControllerEClock,
    parallel_io_controller_e,
    13,
    GMACClock,
    gmac,
    44,
    UART1Clock,
    uart_1,
    45,
);

#[cfg(feature = "atsam4s")]
peripheral_clocks!(
    UART0Clock,
    uart_0,
    8,
    UART1Clock,
    uart_1,
    9,
    StaticMemoryControllerClock,
    static_memory_controller,
    10,
    ParallelIOControllerAClock,
    parallel_io_controller_a,
    11,
    ParallelIOControllerBClock,
    parallel_io_controller_b,
    12,
    ParallelIOControllerCClock,
    parallel_io_controller_c,
    13,
);

#[derive(Default)]
pub struct ClockController {
    pub peripheral_clocks: PeripheralClocks,
}

impl ClockController {
    pub fn new() -> Self {
        ClockController {
            peripheral_clocks: PeripheralClocks::new(),
        }
    }
}
