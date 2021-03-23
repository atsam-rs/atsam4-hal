extern crate nb;

// device crate
use {
    paste::paste,
    core::marker::PhantomData,

    crate::pac::{
        UART0,
        UART1,
    },
    crate::clock::{
        UART0Clock,
        UART1Clock,
        Enabled,
    },
    crate::gpio::{ Pa9, Pa10, PfA },
    crate::time::{ Bps },

    embedded_hal::{serial::Read, serial::Write}
};

#[cfg(feature = "atsam4s")]
use {
    crate::gpio::{ Pb2, Pb3 },
};

#[cfg(feature = "atsam4e")]
use {
    crate::gpio::{ Pa5, Pa6, PfC },
};

#[derive(Debug)]
pub enum Parity {
    Even,
    Odd,
    Space,
    Mark,
}

#[derive(Debug)]
pub enum CharacterLength {
    FiveBits,
    SixBits,
    SevenBits,
    EightBits,
}

#[derive(Debug)]
pub enum StopBits {
    One,
    OnePointFive,
    Two,
}

#[derive(Debug)]
pub enum Error {
    /// Buffer overrun
    Overrun,
    // omitted: other error variants
}

macro_rules! uarts {
    (
        $($PortType:ident: (
            $UART:ident,
            $uart:ident,
            $pin_rx:ty,
            $pin_tx:ty
        ),)+
    ) => {
        paste! {
            $(
                pub struct $PortType { 
                    uart: $UART,
                    clock: PhantomData<[<$UART Clock>]<Enabled>>,
                    rx_pin: PhantomData<$pin_rx>,
                    tx_pin: PhantomData<$pin_tx>,
                }

                impl $PortType {
                    pub fn new (
                        mut uart: $UART,
                        clock: [<$UART Clock>]<Enabled>,
                        _rx_pin: $pin_rx,
                        _tx_pin: $pin_tx,
                        baud_rate: Bps,
                        parity: Option<Parity>,
                    ) -> Self {
                        Self::reset_and_disable(&mut uart);

                        let clock_divisor:u32 = (clock.frequency().0 / baud_rate.0) / 16;
                        if !(1..=65535).contains(&clock_divisor) {
                            panic!("Unsupported baud_rate specified for serial device (cd = {})", clock_divisor);
                        }

                        // Configure the baud rate generator
                        uart.brgr.write(|w| unsafe { w.bits(clock_divisor) });

                        // Configure the mode
                        uart.mr.write(|w| unsafe {
                            // parity
                            if let Some(parity) = parity {
                                let p = match parity {
                                    Parity::Even => 0,
                                    Parity::Odd => 1,
                                    Parity::Space => 2,
                                    Parity::Mark => 3,
                                };
                                w.par().bits(p);
                            }
                            else {
                                w.par().bits(4);  // No parity
                            }

                            w.chmode().bits(0) // Normal mode (not loopback)
                        });

                        Self::enable(&mut uart);

                        $PortType {
                            uart,
                            clock: PhantomData,
                            rx_pin: PhantomData,
                            tx_pin: PhantomData,
                        }
                    }

                    fn reset_and_disable(uart: &mut $UART) {
                        uart.cr.write_with_zero(|w| {
                            w.rstrx().set_bit().rsttx().set_bit().rxdis().set_bit().txdis().set_bit()
                        });
                    }

                    fn enable(uart: &mut $UART) {
                        uart.cr.write_with_zero(|w| {
                            w.rxen().set_bit().txen().set_bit()
                        });
                    }

                    pub fn write_string_blocking(&mut self, data: &str) {
                        for c in data.chars() { 
                            loop {
                                if let Err(_e) = self.write(c as u8) {
                                    continue;
                                }
                                
                                break;
                            }
                        }
                    }
                }

                impl Read<u8> for $PortType {
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

                impl Write<u8> for $PortType {
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
            )+
        }
    }
}

#[cfg(feature = "atsam4s")]
uarts! (
    Uart0: (UART0, uart0, Pa9<PfA>, Pa10<PfA>),
    Uart1: (UART1, uart1, Pb2<PfA>, Pb3<PfA>),
);

#[cfg(feature = "atsam4e")]
uarts! (
    Uart0: (UART0, uart0, Pa9<PfA>, Pa10<PfA>),
    Uart1: (UART1, uart1, Pa5<PfC>, Pa6<PfC>),
);

pub type Serial0 = Uart0;
pub type Serial1 = Uart1;