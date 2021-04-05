use crate::hal::timer::{CountDown, Periodic};
use crate::BorrowUnchecked;
use core::marker::PhantomData;
use embedded_time::duration::*;
use embedded_time::rate::*;
use void::Void;

use crate::pac::TC0;
#[cfg(any(feature = "atsam4e_e", feature = "atsam4n_c", feature = "atsam4s_c"))]
use crate::pac::TC1;
#[cfg(feature = "atsam4e_e")]
use crate::pac::TC2;

#[cfg(any(feature = "atsam4e_e", feature = "atsam4n_c", feature = "atsam4s_c"))]
use crate::clock::Tc1Clock;
#[cfg(feature = "atsam4e_e")]
use crate::clock::Tc2Clock;
use crate::clock::{Enabled, Tc0Clock};

#[derive(Clone, Copy, Debug, PartialEq, Eq, defmt::Format)]
pub enum ClockSource {
    MckDiv2 = 0,
    MckDiv8 = 1,
    MckDiv32 = 2,
    MckDiv128 = 3,
    Slck32768Hz = 4,
}

/// Hardware timers for atsam4 can be 16 or 32-bit
/// depending on the hardware..
/// It is also possible to chain TC (timer channels)
/// within a Timer Module to create larger timer
/// registers (not currently implemented in this hal).
/// TimerCounter implements both the `Periodic` and
/// the `CountDown` embedded_hal timer traits.
/// Before a hardware timer can be used, it must first
/// have a clock configured.
pub struct TimerCounter<TC, CLK> {
    clock: CLK,
    _tc: TC,
}

pub struct TimerCounterChannels<TC> {
    pub ch0: TimerCounterChannel<TC, 0>,
    pub ch1: TimerCounterChannel<TC, 1>,
    pub ch2: TimerCounterChannel<TC, 2>,
}

pub struct TimerCounterChannel<TC, const CH: usize> {
    freq: Hertz,
    source: ClockSource,
    _mode: PhantomData<TC>,
}

macro_rules! tc {
    ($($TYPE:ident: ($TC:ident, $clock:ident),)+) => {
        $(
pub type $TYPE = TimerCounter<$TC, $clock<Enabled>>;

impl TimerCounter<$TC, $clock<Enabled>>
{
    /// Configure this timer counter block.
    /// Each TC block has 3 channels
    /// The clock is obtained from the `ClockController` instance
    /// and its frequency impacts the resolution and maximum range of
    /// the timeout values that can be passed to the `start` method.
    ///
    /// Example
    /// ```
    /// let clocks = ClockController::new(
    ///     cx.device.PMC,
    ///     &cx.device.SUPC,
    ///     &cx.device.EFC0,
    ///     MainClock::Crystal12Mhz,
    ///     SlowClock::RcOscillator32Khz,
    /// );
    ///
    /// let mut tc0 = TimerCounter::new(TC0, clocks.peripheral_clocks.tc_0.into_enabled_clock());
    /// let tc0_chs = tc0.split();
    ///
    /// let mut tcc0 = tc0_chs.ch0;
    /// tcc0.clock_input(ClockSource::Slck32768Hz);
    /// tcc0.start(500_000_000u32.nanoseconds());
    /// while !tcc0.wait().is_ok() {}
    ///
    /// let mut tcc1 = tc0_chs.ch1;
    /// tcc1.clock_input(ClockSource::MckDiv2);
    /// tcc1.start(17u32.nanoseconds()); // Assuming MCK is 120 MHz or faster
    /// while !tcc1.wait().is_ok() {}
    /// ```
    pub fn new(tc: $TC, clock: $clock<Enabled>) -> Self {
        // Disable write-protect mode
        tc.wpmr.write_with_zero(|w| w.wpkey().passwd().wpen().clear_bit());

        // Disable timer channels while reconfiguring
        tc.ccr0.write_with_zero(|w| w.clkdis().set_bit());
        tc.ccr1.write_with_zero(|w| w.clkdis().set_bit());
        tc.ccr2.write_with_zero(|w| w.clkdis().set_bit());

        Self {
            clock,
            _tc: tc,
        }
    }

    /// Splits the TimerCounter module into 3 channels
    /// Defaults to MckDiv2 clock source
    pub fn split(self) -> TimerCounterChannels<$TC> {
        let freq = self.clock.frequency();
        let source = ClockSource::MckDiv2;
        TimerCounterChannels {
            ch0: TimerCounterChannel { freq, source, _mode: PhantomData },
            ch1: TimerCounterChannel { freq, source, _mode: PhantomData },
            ch2: TimerCounterChannel { freq, source, _mode: PhantomData },
        }
    }
}

impl<const CH: usize> TimerCounterChannel<$TC, CH> {
    /// Set the input clock
    pub fn clock_input(&mut self, source: ClockSource) {
        self.source = source;

        // Setup divider
        match CH {
            0 => $TC::borrow_unchecked(|tc| tc.cmr0().modify(|_, w| w.tcclks().bits(source as u8))),
            1 => $TC::borrow_unchecked(|tc| tc.cmr1().modify(|_, w| w.tcclks().bits(source as u8))),
            2 => $TC::borrow_unchecked(|tc| tc.cmr2().modify(|_, w| w.tcclks().bits(source as u8))),
            _ => panic!("Invalid TimerCounterChannel: {}", CH),
        }
    }

    /// Enable the interrupt for this TimerCounterChannel
    /// NOTE: The interrupt used will be TC * 3 + CH
    ///       e.g. TC:1 CH:2 => 1 * 3 + 2 = 5
    pub fn enable_interrupt(&mut self) {
        match CH {
            0 => $TC::borrow_unchecked(|tc| tc.ier0.write_with_zero(|w| w.cpcs().set_bit())),
            1 => $TC::borrow_unchecked(|tc| tc.ier1.write_with_zero(|w| w.cpcs().set_bit())),
            2 => $TC::borrow_unchecked(|tc| tc.ier2.write_with_zero(|w| w.cpcs().set_bit())),
            _ => panic!("Invalid TimerCounterChannel: {}", CH),
        }
    }

    /// Disables the interrupt for this TimerCounterChannel
    pub fn disable_interrupt(&mut self) {
        match CH {
            0 => $TC::borrow_unchecked(|tc| tc.idr0.write_with_zero(|w| w.cpcs().set_bit())),
            1 => $TC::borrow_unchecked(|tc| tc.idr1.write_with_zero(|w| w.cpcs().set_bit())),
            2 => $TC::borrow_unchecked(|tc| tc.idr2.write_with_zero(|w| w.cpcs().set_bit())),
            _ => panic!("Invalid TimerCounterChannel: {}", CH),
        }
    }

    /// Clear interrupt status
    pub fn clear_interrupt_flags(&mut self) -> bool {
        match CH {
            0 => $TC::borrow_unchecked(|tc| tc.sr0.read().cpcs().bit()),
            1 => $TC::borrow_unchecked(|tc| tc.sr1.read().cpcs().bit()),
            2 => $TC::borrow_unchecked(|tc| tc.sr2.read().cpcs().bit()),
            _ => panic!("Invalid TimerCounterChannel: {}", CH),
        }
    }
}
impl<const CH: usize> Periodic for TimerCounterChannel<$TC, CH> {}
impl<const CH: usize> CountDown for TimerCounterChannel<$TC, CH> {
    type Time = Nanoseconds;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Self::Time>,
    {
        // Determine the cycle count
        let timeout: Nanoseconds = timeout.into();
        let rate: Hertz = timeout.to_rate().unwrap();

        let src_freq = match self.source {
            ClockSource::MckDiv2 => self.freq / 2,
            ClockSource::MckDiv8 => self.freq / 8,
            ClockSource::MckDiv32 => self.freq / 32,
            ClockSource::MckDiv128 => self.freq / 128,
            ClockSource::Slck32768Hz => 32768_u32.Hz(),
        };

        // Check if timeout is too fast
        if rate > src_freq {
            panic!("{} Hz is too fast. Max {} Hz.", rate, src_freq);
        }

        // atsam4e supports 32-bits clock timers
        #[cfg(feature = "atsam4e")]
        let max_counter = u32::max_value();
        // atsam4n and atsam4s support 16-bit clock timers
        #[cfg(any(feature = "atsam4n", feature = "atsam4s"))]
        let max_counter = u16::max_value();

        // Compute cycles
        let cycles = src_freq.0 / rate.0;

        // Check if timeout too slow
        if cycles > max_counter.into() {
            let min_freq = src_freq / max_counter.into();
            panic!("{} Hz is too slow. Min {} Hz.", rate, min_freq);
        }

        defmt::trace!("{}->{} Cycles:{} ClockSource:{}", core::stringify!($TC), CH, cycles, self.source);

        // Setup divider
        match CH {
            0 => $TC::borrow_unchecked(|tc| tc.cmr0().modify(|_, w| w.tcclks().bits(self.source as u8).cpctrg().set_bit())),
            1 => $TC::borrow_unchecked(|tc| tc.cmr1().modify(|_, w| w.tcclks().bits(self.source as u8).cpctrg().set_bit())),
            2 => $TC::borrow_unchecked(|tc| tc.cmr2().modify(|_, w| w.tcclks().bits(self.source as u8).cpctrg().set_bit())),
            _ => panic!("Invalid TimerCounterChannel: {}", CH),
        }

        // Setup count-down value
        match CH {
            0 => $TC::borrow_unchecked(|tc| tc.rc0.write_with_zero(|w| unsafe { w.rc().bits(cycles) })),
            1 => $TC::borrow_unchecked(|tc| tc.rc1.write_with_zero(|w| unsafe { w.rc().bits(cycles) })),
            2 => $TC::borrow_unchecked(|tc| tc.rc2.write_with_zero(|w| unsafe { w.rc().bits(cycles) })),
            _ => panic!("Invalid TimerCounterChannel: {}", CH),
        }

        // Clear the interrupt status
        self.clear_interrupt_flags();

        // Enable timer and start using software trigger
        match CH {
            0 => $TC::borrow_unchecked(|tc| tc.ccr0.write_with_zero(|w| w.clken().set_bit().swtrg().set_bit())),
            1 => $TC::borrow_unchecked(|tc| tc.ccr1.write_with_zero(|w| w.clken().set_bit().swtrg().set_bit())),
            2 => $TC::borrow_unchecked(|tc| tc.ccr2.write_with_zero(|w| w.clken().set_bit().swtrg().set_bit())),
            _ => panic!("Invalid TimerCounterChannel: {}", CH),
        }
    }

    fn wait(&mut self) -> nb::Result<(), Void> {
        if match CH {
            0 => $TC::borrow_unchecked(|tc| tc.sr0.read().cpcs().bit()),
            1 => $TC::borrow_unchecked(|tc| tc.sr1.read().cpcs().bit()),
            2 => $TC::borrow_unchecked(|tc| tc.sr2.read().cpcs().bit()),
            _ => panic!("Invalid TimerCounterChannel: {}", CH),
        } {
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}
        )+
    }
}

tc! {
    TimerCounter0: (TC0, Tc0Clock),
}

#[cfg(any(feature = "atsam4e_e", feature = "atsam4n_c", feature = "atsam4s_c"))]
tc! {
    TimerCounter1: (TC1, Tc1Clock),
}

#[cfg(feature = "atsam4e_e")]
tc! {
    TimerCounter2: (TC2, Tc2Clock),
}
