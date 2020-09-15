//! General Purpose Input / Output

use core::marker::PhantomData;

/// Extension trait to split a GPIO peripheral in independent pins and registers
pub trait GpioExt {
    /// The to split the GPIO into
    type Parts;

    /// Splits the GPIO block into independent pins and registers
    fn split(self) -> Self::Parts;
}

/// Input mode (type state)
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating input (type state)
pub struct Floating;
/// Pulled down input (type state)
pub struct PullDown;
/// Pulled up input (type state)
pub struct PullUp;

/// Output mode (type state)
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Push pull output (type state)
pub struct PushPull;
/// Open drain output (type state)
pub struct OpenDrain;

/// Peripheral function A (type state)
pub struct PFA;

/// Peripheral function B (type state)
// pub struct PFB;

// /// Peripheral function C (type state)
// pub struct PFC;

// /// Peripheral function D (type state)
// pub struct PFD;

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $PXx:ident, [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
    ]) => {
        pub mod $gpiox {
            use core::marker::PhantomData;

            use embedded_hal::digital::v2::OutputPin;
            use crate::pac::{$gpiox, $GPIOX};

            use super::{
                PFA, //PFB, PFC, PFD,
                GpioExt,
                Floating, Input, OpenDrain, Output, PullDown, PullUp, PushPull,
            };

            /// Opaque port reference
            pub struct Port {
                _0: (),
            }

            impl Port {
                pub(crate) fn puer(&mut self) -> &$gpiox::PUER {
                    unsafe { &(*$GPIOX::ptr()).puer }
                }

                pub(crate) fn pudr(&mut self) -> &$gpiox::PUDR {
                    unsafe { &(*$GPIOX::ptr()).pudr }
                }

                pub(crate) fn ppder(&mut self) -> &$gpiox::PPDER {
                    unsafe { &(*$GPIOX::ptr()).ppder }
                }

                pub(crate) fn ppddr(&mut self) -> &$gpiox::PPDDR {
                    unsafe { &(*$GPIOX::ptr()).ppddr }
                }

                pub(crate) fn abcdsr1(&mut self) -> &$gpiox::ABCDSR {
                    unsafe { &(*$GPIOX::ptr()).abcdsr[0] }
                }

                pub(crate) fn abcdsr2(&mut self) -> &$gpiox::ABCDSR {
                    unsafe { &(*$GPIOX::ptr()).abcdsr[0] }
                }

                pub(crate) fn idr(&mut self) -> &$gpiox::IDR {
                    unsafe { &(*$GPIOX::ptr()).idr }
                }

                pub(crate) fn mder(&mut self) -> &$gpiox::MDER {
                    unsafe { &(*$GPIOX::ptr()).mder }
                }

                pub(crate) fn mddr(&mut self) -> &$gpiox::MDDR {
                    unsafe { &(*$GPIOX::ptr()).mddr }
                }

                pub(crate) fn oer(&mut self) -> &$gpiox::OER {
                    unsafe { &(*$GPIOX::ptr()).oer }
                }

                pub(crate) fn per(&mut self) -> &$gpiox::PER {
                    unsafe { &(*$GPIOX::ptr()).per }
                }
            }

            /// GPIO parts
            pub struct Parts {
                pub port: Port,
                $(
                    /// Pin
                    pub $pxi: $PXi<$MODE>,
                )+
            }

            impl GpioExt for $GPIOX {
                type Parts = Parts;

                fn split(self) -> Parts {
                    // BUGBUG: Reset the device?
                    // ahb.enr().modify(|_, w| w.$iopxenr().enabled());
                    // ahb.rstr().modify(|_, w| w.$iopxrst().set_bit());
                    // ahb.rstr().modify(|_, w| w.$iopxrst().clear_bit());

                    Parts {
                        port: Port { _0: () },
                        $(
                            $pxi: $PXi { _mode: PhantomData },
                        )+
                    }
                }
            }
            
            /// Partially erased pin
            pub struct $PXx<MODE> {
                i: u8,
                _mode: PhantomData<MODE>,
            }

            impl<MODE> OutputPin for $PXx<Output<MODE>> {
                type Error = ();

                fn set_high(&mut self) -> Result<(), Self::Error> {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*$GPIOX::ptr()).sodr.write_with_zero(|w| w.bits(1 << self.i)) }
                    Ok(())
                }

                fn set_low(&mut self) -> Result<(), Self::Error> {
                    // NOTE(unsafe) atomic write to a stateless register
                    unsafe { (*$GPIOX::ptr()).codr.write_with_zero(|w| w.bits(1 << self.i)) }
                    Ok(())
                }
            }

            $(
                /// Pin
                pub struct $PXi<MODE> {
                    _mode: PhantomData<MODE>,
                }

                impl<MODE> $PXi<MODE> {
                    /// Configures the pin to serve as peripheral function A (PFA)
                    pub fn into_pfa(
                        self,
                        port: &mut Port,
                    ) -> $PXi<PFA> {
                        port.abcdsr1().modify(|r, w| unsafe { w.bits(r.bits() & !(1 << $i)) });
                        port.abcdsr2().modify(|r, w| unsafe { w.bits(r.bits() & !(1 << $i)) });

                        $PXi { _mode: PhantomData }
                    }

                    // BUGBUG: Rest of peripheral functions here.

                    /// Configures the pin to operate as a floating input pin
                    pub fn into_floating_input(
                        self,
                        port: &mut Port,
                    ) -> $PXi<Input<Floating>> {
                        port.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
                        port.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled down input pin
                    pub fn into_pull_down_input(
                        self,
                        port: &mut Port,
                    ) -> $PXi<Input<PullDown>> {
                        port.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) });  // disable pull-up (this must happen first when enabling pull-down resistors)
                        port.ppder().write_with_zero(|w| unsafe { w.bits(1 << $i) });  // enable pull-down

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled up input pin
                    pub fn into_pull_up_input(
                        self,
                        port: &mut Port,
                    ) -> $PXi<Input<PullUp>> {
                        port.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
                        port.puer().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an open drain output pin
                    pub fn into_open_drain_output(
                        self,
                        port: &mut Port,
                    ) -> $PXi<Output<OpenDrain>> {
                        // Disable interrupts for pin
                        port.idr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Enable open-drain/multi-drive
                        port.mder().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Enable output mode
                        port.oer().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Enable pio mode (disables peripheral control of pin)
                        port.per().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        $PXi { _mode: PhantomData }
                    }

                    // Configures the pin to operate as an push pull output pin
                    pub fn into_push_pull_output(
                         self,
                         port: &mut Port,
                    ) -> $PXi<Output<PushPull>> {
                        // Disable interrupts for pin
                        port.idr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Disable open-drain/multi-drive
                        port.mddr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Enable output mode
                        port.oer().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Enable pio mode (disables peripheral control of pin)
                        port.per().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        $PXi { _mode: PhantomData }
                    }
                }

                // impl $PXi<Output<OpenDrain>> {
                //     /// Enables / disables the internal pull up
                //     pub fn internal_pull_up(&mut self, pupdr: &mut PUPDR, on: bool) {
                //         let offset = 2 * $i;

                //         pupdr.pupdr().modify(|r, w| unsafe {
                //             w.bits(
                //                 (r.bits() & !(0b11 << offset)) | if on {
                //                     0b01 << offset
                //                 } else {
                //                     0
                //                 },
                //             )
                //         });
                //     }
                // }

                // impl<MODE> $PXi<Output<MODE>> {
                //     /// Erases the pin number from the type
                //     ///
                //     /// This is useful when you want to collect the pins into an array where you
                //     /// need all the elements to have the same type
                //     pub fn downgrade(self) -> $PXx<Output<MODE>> {
                //         $PXx {
                //             i: $i,
                //             _mode: self._mode,
                //         }
                //     }
                // }

                impl<MODE> OutputPin for $PXi<Output<MODE>> {
                    type Error = ();

                    fn set_high(&mut self) -> Result<(), Self::Error> {
                        // NOTE(unsafe) atomic write to a stateless register
                        unsafe { (*$GPIOX::ptr()).sodr.write_with_zero(|w| w.bits(1 << $i)) }
                        Ok(())
                    }

                    fn set_low(&mut self) -> Result<(), Self::Error> {
                        // NOTE(unsafe) atomic write to a stateless register
                        unsafe { (*$GPIOX::ptr()).codr.write_with_zero(|w| w.bits(1 << $i)) }
                        Ok(())
                    }
                }
            )+
        }
    }
}

gpio!(PIOA, pioa, PAx, [
    PA0: (pa0, 0, Input<Floating>),
    PA1: (pa1, 1, Input<Floating>),
    PA2: (pa2, 2, Input<Floating>),
    PA3: (pa3, 3, Input<Floating>),
    PA4: (pa4, 4, Input<Floating>),
    PA5: (pa5, 5, Input<Floating>),
    PA6: (pa6, 6, Input<Floating>),
    PA7: (pa7, 7, Input<Floating>),
    PA8: (pa8, 8, Input<Floating>),
    PA9: (pa9, 9, Input<Floating>),

    PA10: (pa10, 10, Input<Floating>),
    PA11: (pa11, 11, Input<Floating>),
    PA12: (pa12, 12, Input<Floating>),
    PA13: (pa13, 13, Input<Floating>),
    PA14: (pa14, 14, Input<Floating>),
    PA15: (pa15, 15, Input<Floating>),
    PA16: (pa16, 16, Input<Floating>),
    PA17: (pa17, 17, Input<Floating>),
    PA18: (pa18, 18, Input<Floating>),
    PA19: (pa19, 19, Input<Floating>),

    PA20: (pa20, 20, Input<Floating>),
    PA21: (pa21, 21, Input<Floating>),
    PA22: (pa22, 22, Input<Floating>),
    PA23: (pa23, 23, Input<Floating>),
    PA24: (pa24, 24, Input<Floating>),
    PA25: (pa25, 25, Input<Floating>),
    PA26: (pa26, 26, Input<Floating>),
    PA27: (pa27, 27, Input<Floating>),
    PA28: (pa28, 28, Input<Floating>),
    PA29: (pa29, 29, Input<Floating>),

    PA30: (pa30, 30, Input<Floating>),
    PA31: (pa31, 31, Input<Floating>),
]);

gpio!(PIOB, piob, PBx, [
    PB0: (pb0, 0, Input<Floating>),
    PB1: (pb1, 1, Input<Floating>),
    PB2: (pb2, 2, Input<Floating>),
    PB3: (pb3, 3, Input<Floating>),
    PB4: (pb4, 4, Input<Floating>),
    PB5: (pb5, 5, Input<Floating>),
    PB6: (pb6, 6, Input<Floating>),
    PB7: (pb7, 7, Input<Floating>),
    PB8: (pb8, 8, Input<Floating>),
    PB9: (pb9, 9, Input<Floating>),

    PB10: (pb10, 10, Input<Floating>),
    PB11: (pb11, 11, Input<Floating>),
    PB12: (pb12, 12, Input<Floating>),
    PB13: (pb13, 13, Input<Floating>),
    PB14: (pb14, 14, Input<Floating>),

    // PB15-31 do not exist.
]);

gpio!(PIOC, pioc, PCx, [
    PC0: (pc0, 0, Input<Floating>),
    PC1: (pc1, 1, Input<Floating>),
    PC2: (pc2, 2, Input<Floating>),
    PC3: (pc3, 3, Input<Floating>),
    PC4: (pc4, 4, Input<Floating>),
    PC5: (pc5, 5, Input<Floating>),
    PC6: (pc6, 6, Input<Floating>),
    PC7: (pc7, 7, Input<Floating>),
    PC8: (pc8, 8, Input<Floating>),
    PC9: (pc9, 9, Input<Floating>),

    PC10: (pc10, 10, Input<Floating>),
    PC11: (pc11, 11, Input<Floating>),
    PC12: (pc12, 12, Input<Floating>),
    PC13: (pc13, 13, Input<Floating>),
    PC14: (pc14, 14, Input<Floating>),
    PC15: (pc15, 15, Input<Floating>),
    PC16: (pc16, 16, Input<Floating>),
    PC17: (pc17, 17, Input<Floating>),
    PC18: (pc18, 18, Input<Floating>),
    PC19: (pc19, 19, Input<Floating>),

    PC20: (pc20, 20, Input<Floating>),
    PC21: (pc21, 21, Input<Floating>),
    PC22: (pc22, 22, Input<Floating>),
    PC23: (pc23, 23, Input<Floating>),
    PC24: (pc24, 24, Input<Floating>),
    PC25: (pc25, 25, Input<Floating>),
    PC26: (pc26, 26, Input<Floating>),
    PC27: (pc27, 27, Input<Floating>),
    PC28: (pc28, 28, Input<Floating>),
    PC29: (pc29, 29, Input<Floating>),

    PC30: (pc30, 30, Input<Floating>),
    PC31: (pc31, 31, Input<Floating>),
]);

gpio!(PIOD, piod, PDx, [
    PD0: (pd0, 0, Input<Floating>),
    PD1: (pd1, 1, Input<Floating>),
    PD2: (pd2, 2, Input<Floating>),
    PD3: (pd3, 3, Input<Floating>),
    PD4: (pd4, 4, Input<Floating>),
    PD5: (pd5, 5, Input<Floating>),
    PD6: (pd6, 6, Input<Floating>),
    PD7: (pd7, 7, Input<Floating>),
    PD8: (pd8, 8, Input<Floating>),
    PD9: (pd9, 9, Input<Floating>),

    PD10: (pd10, 10, Input<Floating>),
    PD11: (pd11, 11, Input<Floating>),
    PD12: (pd12, 12, Input<Floating>),
    PD13: (pd13, 13, Input<Floating>),
    PD14: (pd14, 14, Input<Floating>),
    PD15: (pd15, 15, Input<Floating>),
    PD16: (pd16, 16, Input<Floating>),
    PD17: (pd17, 17, Input<Floating>),
    PD18: (pd18, 18, Input<Floating>),
    PD19: (pd19, 19, Input<Floating>),

    PD20: (pd20, 20, Input<Floating>),
    PD21: (pd21, 21, Input<Floating>),
    PD22: (pd22, 22, Input<Floating>),
    PD23: (pd23, 23, Input<Floating>),
    PD24: (pd24, 24, Input<Floating>),
    PD25: (pd25, 25, Input<Floating>),
    PD26: (pd26, 26, Input<Floating>),
    PD27: (pd27, 27, Input<Floating>),
    PD28: (pd28, 28, Input<Floating>),
    PD29: (pd29, 29, Input<Floating>),

    PD30: (pd30, 30, Input<Floating>),
    PD31: (pd31, 31, Input<Floating>),
]);
