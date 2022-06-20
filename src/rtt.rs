use crate::hal::timer::{CountDown, Periodic};
use crate::pac::RTT;
use embedded_time::duration::*;
use embedded_time::rate::*;
use void::Void;

/// RTT (Real-time Timer) can be configured in one of
/// two ways:
/// 1. Use 32.768 kHz (/w 16-bit prescaler) input clock
///    to expire a 32-bit counter. The prescaler has an additional
///    interrupt that can be triggered on incrementing.
///    input clock to expire a 32-bit counter.
/// 2. Use 1 Hz RC clock, 16-bit prescaler is ignored and can be used
///    separately. This requires the RTC module is setup and enabled.
///
/// (1) is independent of (2), except that the 16-bit prescaler is shared.
pub struct RealTimeTimer {
    rtt: RTT,
    rtc1hz: bool, // rtc1hz is write-only, so we have to store it
}

impl Periodic for RealTimeTimer {}
impl CountDown for RealTimeTimer {
    type Time = Microseconds;

    fn start<T>(&mut self, timeout: T)
    where
        T: Into<Self::Time>,
    {
        // Disable timer during configuration
        self.rtt.mr.modify(|_, w| w.rttdis().set_bit());

        // Check if ALMIEN is set (need to disable, then re-enable)
        let rtt_mr = self.rtt.mr.read();
        let almien = rtt_mr.almien().bit_is_set();
        let rttincien = rtt_mr.rttincien().bit_is_set();

        // Determine set prescaler
        let prescaler = self.rtt.mr.read().rtpres().bits();

        // Calculate the prescaler period
        let period: Microseconds = if self.rtc1hz {
            1_u32.Hz().to_duration().unwrap()
        } else {
            let slck_duration: Microseconds = 32_768_u32.Hz().to_duration().unwrap();
            match prescaler {
                0 => slck_duration * 2_u32.pow(16),
                1 | 2 => 1_u32.Hz().to_duration().unwrap(), // Invalid
                _ => slck_duration * prescaler.into(),
            }
        };

        // Determine alarm value
        let timeout: u32 = timeout.into().integer();
        let period: u32 = period.integer();
        let alarmv = timeout / period;

        // ALMIEN must be disabled when setting a new alarm value
        if almien {
            self.disable_alarm_interrupt();
        }
        if rttincien {
            self.disable_prescaler_interrupt();
        }

        // The alarm value is always alarmv - 1 as RTT_AR is set
        // to 0xFFFF_FFFF on reset
        self.rtt.ar.write(|w| unsafe { w.almv().bits(alarmv) });

        // Re-enable ALMIEN if it was enabled
        if almien {
            self.enable_alarm_interrupt();
        }
        if rttincien {
            self.enable_prescaler_interrupt();
        }

        // Start timer, making sure to start fresh
        // NOTE: This seems to behave better as two calls when prescaler is set to 3
        self.rtt.mr.modify(|_, w| w.rttdis().clear_bit());
        self.rtt.mr.modify(|_, w| w.rttrst().set_bit());
    }

    /// Waits on the 32-bit register alarm flag (ALMS)
    fn wait(&mut self) -> nb::Result<(), Void> {
        // Reading clears the flag, so store it for analysis
        // Double-reading can cause interesting issues where the module
        // doesn't reset the timer correctly.
        let rtt_sr = self.rtt.sr.read();

        // Reading clears the flag
        if rtt_sr.alms().bit_is_set() {
            // Reset the timer (to ensure we're periodic)
            self.rtt.mr.modify(|_, w| w.rttrst().set_bit());
            Ok(())
        } else {
            Err(nb::Error::WouldBlock)
        }
    }
}

impl RealTimeTimer {
    /// RTT is simple to initialize as it requires no other setup.
    /// (with the exception of using a 32.768 kHz crystal).
    /// Both the internal RC counters (32.768 kHz and 1 Hz) require
    /// no setup.
    ///
    /// If prescaler is equal to zero, the prescaler period
    /// is equal to 2^16 * SCLK period. If not, the prescaler period
    /// is equal to us_prescaler * SCLK period.
    /// 0         - 2^16 * SCLK
    /// 1, 2      - Forbidden
    /// Otherwise - RTPRES * SLCK
    /// 3 => 32.768 kHz / 3 = 10.92267 kHz (91552.706 ns)
    /// This means our minimum unit of time is ~92 us.
    ///
    /// The maximum amount of time using the minimum unit of time:
    /// 91552.706 ns * 2^32 = 3.932159E14
    ///  393215.9     seconds
    ///    6553.598   minutes
    ///     109.2266  hours
    ///       4.55111 days
    ///
    /// If the rtc1hz is enabled, a 1 Hz signal is used for the 32-bit
    /// alarm. The prescaler is still active and can be triggered from
    /// the prescaler increment interrupt.
    ///
    /// ```rust
    /// let prescaler = 3;
    /// let rtc1hz = false;
    /// let mut rtt = RealTimeTimer::new(peripherals.RTT, prescaler, rtc1hz);
    /// // Set Wait for 1 second
    /// rtt.start(1_000_000u32.microseconds());
    /// // Wait for 1 second
    /// while !rtt.wait().is_ok() {}
    /// // Wait for 1 second again
    /// while !rtt.wait().is_ok() {}
    /// ```
    pub fn new(rtt: RTT, prescaler: u16, rtc1hz: bool) -> Self {
        // Panic if prescaler set to 1 or 2
        match prescaler {
            1 | 2 => panic!("RTT prescaler cannot be set to 1 or 2"),
            _ => {}
        }

        // Disable timer while reconfiguring and prescaler interrupt before setting RTPRES
        rtt.mr
            .modify(|_, w| w.rttdis().set_bit().rttincien().clear_bit());

        // Set the prescalar, rtc1hz and reset the prescaler
        // NOTE: rtc1hz is write-only on some MCUs
        rtt.mr.modify(|_, w| unsafe {
            w.rtpres()
                .bits(prescaler)
                .rtc1hz()
                .bit(rtc1hz)
                .rttrst()
                .set_bit()
        });

        Self { rtt, rtc1hz }
    }

    /// Enable the interrupt generation for the 32-bit register
    /// alarm. This method only sets the clock configuration to
    /// trigger the interrupt; it does not configure the interrupt
    /// controller or define an interrupt handler.
    pub fn enable_alarm_interrupt(&mut self) {
        self.rtt.mr.modify(|_, w| w.almien().set_bit());
    }

    /// Enable the interrupt generation for the 16-bit prescaler
    /// overflow. This method only sets the clock configuration to
    /// trigger the interrupt; it does not configure the interrupt
    /// controller or define an interrupt handler.
    pub fn enable_prescaler_interrupt(&mut self) {
        self.rtt.mr.modify(|_, w| w.rttincien().set_bit());
    }

    /// Disables interrupt generation for the 32-bit register alarm.
    /// This method only sets the clock configuration to prevent
    /// triggering the interrupt; it does not configure the interrupt
    /// controller.
    pub fn disable_alarm_interrupt(&mut self) {
        self.rtt.mr.modify(|_, w| w.almien().clear_bit());
    }

    /// Disables interrupt generation for the 16-bit prescaler overflow.
    /// This method only sets the clock configuration to prevent
    /// triggering the interrupt; it does not configure the interrupt
    /// controller.
    pub fn disable_prescaler_interrupt(&mut self) {
        self.rtt.mr.modify(|_, w| w.rttincien().clear_bit());
    }

    /// Clear interrupt status
    /// This will clear both prescaler and alarm interrupts
    pub fn clear_interrupt_flags(&mut self) {
        let _rtt_sr = self.rtt.sr.read();

        // Reset the timer (to ensure we're periodic)
        self.rtt.mr.modify(|_, w| w.rttrst().set_bit());
    }
}
