// See Chapter 28 of ATSAM4 Datasheet
use crate::pac::{EFC, PMC};

// pub enum Oscillator {
//     OSC_SLCK_32K_RC,
//     OSC_SLCK_32K_XTAL,
//     OSC_SLCK_32K_BYPASS,
//     OSC_MAINCK_4M_RC,
//     OSC_MAINCK_8M_RC,
//     OSC_MAINCK_12M_RC,
//     OSC_MAINCK_XTAL,
//     OSC_MAINCK_BYPASS,
// }

// impl Oscillator {
//     pub fn enable() {

//     }
// }

pub struct ClockController {
}

impl ClockController {
    pub fn with_internal_32kosc(pmc: PMC, efc: &mut EFC) -> Self {
        Self::new(pmc, efc, false)
    }

    pub fn with_external_32kosc(pmc: PMC, efc: &mut EFC) -> Self {
        Self::new(pmc, efc, true)
    }

    pub fn new(pmc: PMC, efc: &mut EFC, use_external_oscillator: bool) -> Self {
        Self::set_flash_wait_states_to_maximum(efc);
        
        ClockController {}
    }

    fn get_flash_wait_states_for_clock_frequency(clock_frequency: usize) -> u8 {
        match clock_frequency {
            c if c < 20000000 => 0,
            c if c < 40000000 => 1,
            c if c < 60000000 => 2,
            c if c < 80000000 => 3,
            c if c < 100000000 => 4,
            c if c < 123000000 => 5,
            _ => panic!("Invalid frequency provided to get_flash_wait_states(): {} ", clock_frequency),
        }
    }

    fn set_flash_wait_states_to_maximum(efc: &mut EFC) {
        efc.fmr.modify(|_, w| w.fws().set(5).cloe.set_bit());
    }

    fn set_flash_wait_states_for_clock_frequency(efc: &mut EFC, clock_frequency: usize) {
        let wait_state_count = Self::get_flash_wait_states_for_clock_frequency(clock_frequency);
        /*
        if clock_frequency < 20000000 {
            wait_states = 0;
        } else if clock_frequency < 40000000 {
            wait_states = 1;
        } else if clock_frequency < 60000000 {
            wait_states = 2;
        } else if clock_frequency < 80000000 {
            wait_states = 3;
        } else if clock_frequency < 100000000 {
            wait_states = 4;
        } else if clock_frequency < 123000000 {
            wait_states = 5;
        } else {
        }
        */

        efc.fmr.modify(|_, w| w.fws().set(wait_state_count).cloe.set_bit());
    }
}
