extern crate nb;

// device crate
use {
    crate::pac::{
        UART0,
    }
};

pub struct Serial<UART> { uart: UART }

// convenience type alias
pub type Serial0 = Serial<UART0>;

/// Serial interface error
pub enum Error {
    /// Buffer overrun
    Overrun,
    // omitted: other error variants
}

impl embedded_hal::serial::Read<u8> for Serial<UART0> {
    type Error = Error;

    fn read(&mut self) -> nb::Result<u8, Error> {
        // read the status register
        let isr = self.uart.sr.read();

        if isr.ovre().bit_is_set() {
            // Error: Buffer overrun
            Err(nb::Error::Other(Error::Overrun))
        }
        // omitted: checks for other errors
        else if isr.rxrdy().bit_is_set() {
            // Data available: read the data register
            Ok(self.uart.rhr.read().bits() as u8)
        } else {
            // No data available yet
            Err(nb::Error::WouldBlock)
        }
    }
}

impl embedded_hal::serial::Write<u8> for Serial<UART0> {
    type Error = Error;

    fn write(&mut self, byte: u8) -> nb::Result<(), Error> {
        // read the status register
        let isr = self.uart.sr.read();

        // omitted: checks for other errors
        if isr.txrdy().bit_is_set() {
            Ok(self.uart.thr.write_with_zero(|w| unsafe { w.txchr().bits(byte) }))
        } else {
            // No data available yet
            Err(nb::Error::WouldBlock)
        }
    }

    fn flush(&mut self) -> nb::Result<(), Error> {
        // No data available yet
        Err(nb::Error::WouldBlock)
    }
}
