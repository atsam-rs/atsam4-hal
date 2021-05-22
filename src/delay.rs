//! Delays
use core::sync::atomic::*;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SYST;
use cortex_m_rt::exception;
pub use hal::blocking::delay::{DelayMs, DelayUs};

use crate::clock::*;

/// System timer (SysTick) as a delay provider
pub struct Delay {
    syst: SYST,
}

impl Delay {
    /// Configures the system timer (SysTick) as a delay provider
    pub fn new(mut syst: SYST) -> Self {
        let total_rvr:u32 = get_master_clock_frequency().0 / 1000;
        syst.set_clock_source(SystClkSource::Core);
        syst.set_reload(total_rvr);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();
    
        Delay {
            syst,
        }
    }

    /// Releases the system timer (SysTick) resource
    pub fn free(self) -> SYST {
        self.syst
    }

    pub fn current_tick() -> u64 {
        SYSTEM_TICK.load(Ordering::Relaxed) as u64
    }
}

impl DelayMs<u64> for Delay {
    fn delay_ms(&mut self, ms: u64) {
        let end = Self::current_tick() + ms;
        while Self::current_tick() < end {}
    }
}

impl DelayMs<u32> for Delay {
    fn delay_ms(&mut self, ms: u32) {
        self.delay_ms(ms as u64);
    }
}

impl DelayMs<u16> for Delay {
    fn delay_ms(&mut self, ms: u16) {
        self.delay_ms(ms as u32);
    }
}

impl DelayMs<u8> for Delay {
    fn delay_ms(&mut self, ms: u8) {
        self.delay_ms(ms as u32);
    }
}

static SYSTEM_TICK: AtomicUsize = AtomicUsize::new(0);

#[exception]
fn SysTick() {
    SYSTEM_TICK.fetch_add(1, Ordering::SeqCst);
}
