// See Chapter 28 of ATSAM4 Datasheet
use crate::pac::{EFC, PMC};
use crate::time::{Hertz};

pub struct ClockController {
    _pmc: PMC,
    master_clock_frequency: Hertz,
}

impl ClockController {
    pub fn with_internal_32kosc(pmc: PMC, efc: &mut EFC) -> Self {
        Self::new(pmc, efc, false)
    }

    pub fn with_external_32kosc(pmc: PMC, efc: &mut EFC) -> Self {
        Self::new(pmc, efc, true)
    }

    pub fn new(mut pmc: PMC, efc: &mut EFC, use_external_oscillator: bool) -> Self {
        Self::set_flash_wait_states_to_maximum(efc);

        if !use_external_oscillator {
            Self::switch_main_clock_to_fast_rc_12mhz(&mut pmc);
        } else {
            panic!("external oscillator not supported")
        }
        
        Self::wait_for_main_clock_ready(&mut pmc);

        // Set up the PLL for 120Mhz operation (12Mhz RC * (10 / 1) = 120Mhz)
        let multiplier:u16 = 10;
        let divider:u8 = 1;
        Self::enable_plla_clock(&mut pmc, multiplier, divider);
        Self::wait_for_plla_lock(&pmc);

        let prescaler = 1;
        Self::switch_master_clock_to_plla(&mut pmc, prescaler);

        let master_clock_frequency = Self::calculate_master_clock_frequency(&pmc);

        Self::set_flash_wait_states_for_clock_frequency(efc, master_clock_frequency);

        ClockController {
            _pmc: pmc,
            master_clock_frequency: master_clock_frequency,
        }
    }

    pub fn get_master_clock_frequency(&self) -> Hertz {
        self.master_clock_frequency
    }

    fn calculate_master_clock_frequency(pmc: &PMC) -> Hertz {
        let mut mclk_freq = match pmc.pmc_mckr.read().css().bits() {
            0 => { // Slow clock
                panic!("Unsupported clock source: Slow clock.")
            },
            1 => { // Main clock
                panic!("Unsupported clock source: Main clock.")
            },
            2 => { // PLL
                let mut mclk_freq:u32 = match pmc.ckgr_mor.read().moscsel().bit_is_set() {
                    true => 12000000,
                    false => {
                        match pmc.ckgr_mor.read().moscrcf().bits() {
                            0 => 4000000,
                            1 => 8000000,
                            2 => 12000000,
                            _ => panic!("Unexpected value detected ready from pmc.ckgr_mor.moscrcf")
                        }
                    }
                };

                let plla_clock_source:u8 = 2; // 2 = PLLA
                if pmc.pmc_mckr.read().css().bits() == plla_clock_source {
                    mclk_freq *= (pmc.ckgr_pllar.read().mula().bits() + 1) as u32;
                    mclk_freq /= (pmc.ckgr_pllar.read().diva().bits()) as u32;
                }

                mclk_freq
            }
            _ => panic!("Invalid value found in PMC_MCKR.CSS")
        };

        // Factor in the prescaler
        mclk_freq = match pmc.pmc_mckr.read().pres().bits() {
            7 => mclk_freq / 3,                 // Special case for a 3 prescaler
            prescaler => mclk_freq >> prescaler,
        };

        Hertz(mclk_freq)
    }

    fn get_flash_wait_states_for_clock_frequency(clock_frequency: Hertz) -> u8 {
        match clock_frequency {
            c if c.0 < 20000000 => 0,
            c if c.0 < 40000000 => 1,
            c if c.0 < 60000000 => 2,
            c if c.0 < 80000000 => 3,
            c if c.0 < 100000000 => 4,
            c if c.0 < 123000000 => 5,
            _ => panic!("Invalid frequency provided to get_flash_wait_states(): {} ", clock_frequency.0),
        }
    }

    fn set_flash_wait_states_to_maximum(efc: &mut EFC) {
        efc.fmr.modify(|_, w| unsafe { w.fws().bits(5).cloe().set_bit() });
    }

    fn set_flash_wait_states_for_clock_frequency(efc: &mut EFC, clock_frequency: Hertz) {
        let wait_state_count = Self::get_flash_wait_states_for_clock_frequency(clock_frequency);
        efc.fmr.modify(|_, w| unsafe { w.fws().bits(wait_state_count).cloe().set_bit() });
    }

    fn switch_main_clock_to_fast_rc_12mhz(pmc: &mut PMC) {
        Self::enable_fast_rc_oscillator(pmc);
        Self::wait_for_fast_rc_oscillator_to_stabilize(pmc);
        Self::change_fast_rc_oscillator_to_12_mhz(pmc);
        Self::wait_for_fast_rc_oscillator_to_stabilize(pmc);
        Self::switch_to_fast_rc_oscillator(pmc);
    }

    fn enable_fast_rc_oscillator(pmc: &mut PMC) {
        pmc.ckgr_mor.modify(|_, w| unsafe { w.key().bits(0x37).moscrcen().set_bit() });
    }

    fn change_fast_rc_oscillator_to_12_mhz(pmc: &mut PMC) {
        pmc.ckgr_mor.modify(|_, w| unsafe { w.key().bits(0x37).moscrcf()._12_mhz() });
    }

    fn switch_to_fast_rc_oscillator(pmc: &mut PMC) {
        pmc.ckgr_mor.modify(|_, w| unsafe { w.key().bits(0x37).moscsel().clear_bit() });
    }

    fn wait_for_fast_rc_oscillator_to_stabilize(pmc: &PMC) {
        while pmc.pmc_sr.read().moscrcs().bit_is_clear() {}
    }

    fn is_main_clock_ready(pmc: &PMC) -> bool {
        pmc.pmc_sr.read().moscsels().bit_is_set()
    }

    fn wait_for_main_clock_ready(pmc: &PMC) {
        while !Self::is_main_clock_ready(pmc) {}
    }

    fn enable_plla_clock(pmc: &mut PMC, multiplier: u16, divider: u8) {
        Self::disable_plla_clock(pmc);

        // NOTE: the datasheet indicates the multplier used it MULA + 1 - hence the subtraction when setting the multiplier.
        pmc.ckgr_pllar.modify(|_, w| unsafe { w.one().set_bit().pllacount().bits(0x3f).mula().bits(multiplier - 1).diva().bits(divider) });
    }

    fn disable_plla_clock(pmc: &mut PMC) {
        pmc.ckgr_pllar.modify(|_, w| unsafe { w.one().set_bit().mula().bits(0) });
    }

    fn is_plla_locked(pmc: &PMC) -> bool {
        pmc.pmc_sr.read().locka().bit_is_set()
    }

    fn wait_for_plla_lock(pmc: &PMC) {
        while !Self::is_plla_locked(pmc) {}
    }

    fn switch_master_clock_to_plla(pmc: &mut PMC, prescaler: u8) {
        // Set the master clock prescaler
        pmc.pmc_mckr.modify(|_, w| w.pres().bits(prescaler) );

        Self::wait_for_master_clock_ready(pmc);

        // Set the master clock source to PLLA
        let clock_source : u8 = 2; // 2 = PLLA
        pmc.pmc_mckr.modify(|_, w| unsafe { w.css().bits(clock_source) });

        Self::wait_for_master_clock_ready(pmc);
    }

    fn is_master_clock_ready(pmc: &PMC) -> bool {
        pmc.pmc_sr.read().mckrdy().bit_is_set()
    }

    fn wait_for_master_clock_ready(pmc: &PMC) {
        while !Self::is_master_clock_ready(pmc) {}
    }
}

impl Into<Hertz> for ClockController {
    fn into(self) -> Hertz {
        self.master_clock_frequency
    }
}
