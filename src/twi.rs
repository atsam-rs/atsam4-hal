use core::marker::PhantomData;
use embedded_time::rate::{Baud, Hertz};
use crate::{
    clock::{Enabled, Twi0Clock, Twi1Clock, get_master_clock_frequency},
    gpio::{Pa3, Pa4, Pb4, Pb5, PfA},
    pac::{TWI0, twi0, TWI1, twi1},
};
use paste::paste;

// TWI Mode Markers
pub struct MasterMode;
pub struct SlaveMode;

/// I2C error
#[derive(Debug)]
pub enum Error {
    AddressError,
    ArbitrationLost,
    Nack,
    Overrun,
}

// Helper functions
/// (ckdiv, chdiv, cldiv)
fn clk_dividers(baud: Baud) -> (u8, u8, u8) {
    let pclk = get_master_clock_frequency();
    // t_low  = ((CLDIV x 2^CKDIV) + 4 x t_periph
    // t_high = ((CHDIV x 2^CKDIV) + 4 x t_periph

    // - Calculations taken from ASF -
    let low_level_time_limit = Hertz(384000);
    let twi_clk_divider = 2;
    let twi_clk_calc_argu = 4;
    let twi_clk_div_max = 0xFF;
    let twi_clk_div_min = 7;

    // Low level time not less than 1.3us of I2C Fast Mode.
    let mut ckdiv: u8 = 0;
    if baud.0 > low_level_time_limit.0 {
        // Low level of time fixed for 1.3us.
        let mut cldiv = pclk.0 / (low_level_time_limit.0 * twi_clk_divider) - twi_clk_calc_argu;
        let mut chdiv = pclk.0 / ((baud.0 + (baud.0 - low_level_time_limit.0)) * twi_clk_divider)
            - twi_clk_calc_argu;

        // cldiv must fit in 8 bits, ckdiv must fit in 3 bits
        while (cldiv > twi_clk_div_max) && (ckdiv < twi_clk_div_min) {
            // Increase clock divider
            ckdiv += 1;
            // Divide cldiv value
            cldiv /= twi_clk_divider;
        }
        // chdiv must fit in 8 bits, ckdiv must fit in 3 bits
        while (chdiv > twi_clk_div_max) && (ckdiv < twi_clk_div_min) {
            // Increase clock divider
            ckdiv += 1;
            // Divide cldiv value
            chdiv /= twi_clk_divider;
        }

        (ckdiv, chdiv as u8, cldiv as u8)
    } else {
        let mut c_lh_div = pclk.0 / (baud.0 * twi_clk_divider) - twi_clk_calc_argu;

        // cldiv must fit in 8 bits, ckdiv must fit in 3 bits
        while (c_lh_div > twi_clk_div_max) && (ckdiv < twi_clk_div_min) {
            // Increase clock divider
            ckdiv += 1;
            // Divide cldiv value
            c_lh_div /= twi_clk_divider;
        }

        (ckdiv, c_lh_div as u8, c_lh_div as u8)
    }
}

macro_rules! two_wire_interfaces {
    (
        $($InterfaceType:ident: (
            $TWI:ident,
            $Twi:ident,
            $twi:ident,
            $pin_clock:ty,
            $pin_data:ty
        ),)+
    ) => {
        paste! {
            $(
                pub struct $InterfaceType<MODE> {
                    _mode: PhantomData<MODE>,
                    twi: $TWI,
                    clock: PhantomData<[<$Twi Clock>]<Enabled>>,
                    clock_pin: PhantomData<$pin_clock>,
                    data_pin: PhantomData<$pin_data>,
                }

                impl<MODE> $InterfaceType<MODE> {
                    fn new<F: FnOnce(&$TWI)> (
                        twi: $TWI,
                        _clock: [<$Twi Clock>]<Enabled>,
                        _clock_pin: $pin_clock,
                        _data_pin: $pin_data,
                        baud: Baud,
                        f: F
                    ) -> Self {
                        // Reset TWI
                        twi.cr.write_with_zero(|w| w.swrst().set_bit());

                        // Setup TWI/I2C Clock
                        let (ckdiv, chdiv, cldiv) = clk_dividers(baud);
                        twi.cwgr.write_with_zero(|w| unsafe {
                            w.ckdiv()
                                .bits(ckdiv)
                                .chdiv()
                                .bits(chdiv)
                                .cldiv()
                                .bits(cldiv)
                        });

                        // Disable slave and master modes
                        twi.cr.write_with_zero(|w| w.msdis().set_bit().svdis().set_bit());

                        // Set baud rate
                        let (ckdiv, chdiv, cldiv) = clk_dividers(baud);
                        twi.cwgr.write_with_zero(|w| unsafe {
                            w.ckdiv()
                                .bits(ckdiv)
                                .chdiv()
                                .bits(chdiv)
                                .cldiv()
                                .bits(cldiv)
                        });

                        // Let the caller perform any additional TWI configuration
                        f(&twi);

                        $InterfaceType {
                            _mode: PhantomData,
                            twi,
                            clock: PhantomData,
                            clock_pin: PhantomData,
                            data_pin: PhantomData,
                        }
                    }

                    pub fn read_one(&self) -> nb::Result<u8, ()> {
                        match self.twi.sr.read().rxrdy().bit_is_set() {
                            false => Err(nb::Error::WouldBlock),
                            true => Ok(self.twi.rhr.read().rxdata().bits())
                        }
                    }
                }

                pub struct [<$InterfaceType Builder>] {
                    twi: $TWI,
                    clock: [<$Twi Clock>]<Enabled>,
                    clock_pin: $pin_clock,
                    data_pin: $pin_data,
                    baud: Baud,
                }

                impl [<$InterfaceType Builder>] {
                    pub fn new(                        
                        twi: $TWI,
                        clock: [<$Twi Clock>]<Enabled>,
                        clock_pin: $pin_clock,
                        data_pin: $pin_data,
                        baud: Baud,
                    ) -> Self {
                        [<$InterfaceType Builder>] {
                            twi,
                            clock,
                            clock_pin,
                            data_pin,
                            baud,
                        }
                    }

                    pub fn baud(mut self, baud: Baud) -> Self {
                        self.baud = baud;
                        self
                    }

                    pub fn into_master(self) -> $InterfaceType<MasterMode> {
                        $InterfaceType::<MasterMode>::new(
                            self.twi,
                            self.clock,
                            self.clock_pin,
                            self.data_pin,
                            self.baud,
                            |twi| {
                                // Enable master mode
                                twi.cr.write_with_zero(|w| {
                                    w
                                        .msen().set_bit()   // Enable master mode
                                        .svdis().set_bit()  // Disable slave mode
                                });        
                            }
                        )
                    }

                    pub fn into_slave(self, address: u8) -> $InterfaceType<SlaveMode> {
                        $InterfaceType::<SlaveMode>::new(
                            self.twi,
                            self.clock,
                            self.clock_pin,
                            self.data_pin,
                            self.baud,
                            |twi| {
                                // Enable slave mode
                                twi.cr.write_with_zero(|w| {
                                    w
                                        .msdis().set_bit()  // Disable master mode
                                        .sven().set_bit()   // Enable slave mode
                                });

                                // Set the slave mode address
                                twi.smr.write_with_zero(|w| unsafe {
                                    w.sadr().bits(address)
                                });
                            }
                        )
                    }
                }
            )+
        }
    }
}

two_wire_interfaces!(
    TwoWireInterface0: (TWI0, Twi0, twi0, Pa4<PfA>, Pa3<PfA>),
    TwoWireInterface1: (TWI1, Twi1, twi1, Pb5<PfA>, Pb4<PfA>),
);
