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
pub struct PFB;

/// Peripheral function C (type state)
pub struct PFC;

/// Peripheral function D (type state)
pub struct PFD;

macro_rules! gpio {
    ($GPIOX:ident, $gpiox:ident, $PXx:ident, [
        $($PXi:ident: ($pxi:ident, $i:expr, $MODE:ty),)+
    ]) => {
        pub mod $gpiox {
            use core::marker::PhantomData;

            use hal::digital::v2::OutputPin;
            use crate::pac::{$gpiox, $GPIOX};

            use super::{
                PFA, PFB, PFC, PFD,
                GpioExt,
                Floating, Input, OpenDrain, Output, PullDown, PullUp, PushPull,
            };

            /// GPIO parts
            pub struct Parts {
                /// Opaque PeripheralSelectRegister1
                pub abcdsr1: PeripheralSelectRegister1,
                /// Opaque PeripheralSelectRegister2
                pub abcdsr2: PeripheralSelectRegister2,
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
                        abcdsr1: PeripheralSelectRegister1 { _0: () },
                        abcdsr2: PeripheralSelectRegister2 { _0: () },
                        $(
                            $pxi: $PXi { _mode: PhantomData },
                        )+
                    }
                }
            }

            /// Opaque PullUpDisableRegister (PIO_PUDR)
            pub struct PullUpDisableRegister {
                _0: (),
            }

            impl PullUpDisableRegister {
                pub(crate) fn pudr(&mut self) -> &$gpiox::PUDR {
                    unsafe { &(*$GPIOX::ptr()).pudr }
                }
            }

            /// Opaque PullUpEnableRegister (PIO_PUER)
            pub struct PullUpEnableRegister {
                _0: (),
            }

            impl PullUpEnableRegister {
                pub(crate) fn puer(&mut self) -> &$gpiox::PUER {
                    unsafe { &(*$GPIOX::ptr()).puer }
                }
            }
            
            /// Opaque PadPulldownDisableRegister (PIO_PPDDR)
            pub struct PadPulldownDisableRegister {
                _0: (),
            }

            impl PadPulldownDisableRegister {
                pub(crate) fn ppddr(&mut self) -> &$gpiox::PPDDR {
                    unsafe { &(*$GPIOX::ptr()).ppddr }
                }
            }

            /// Opaque PadPulldownEnableRegister (PIO_PPDER)
            pub struct PadPulldownEnableRegister {
                _0: (),
            }

            impl PadPulldownEnableRegister {
                pub(crate) fn ppder(&mut self) -> &$gpiox::PPDER {
                    unsafe { &(*$GPIOX::ptr()).ppder }
                }
            }

            /// Opaque PeripheralSelectRegister1 (PIO_ABCDSR1)
            pub struct PeripheralSelectRegister1 {
                _0: (),
            }

            impl PeripheralSelectRegister1 {
                pub(crate) fn abcdsr1(&mut self) -> &$gpiox::ABCDSR {
                    unsafe { &(*$GPIOX::ptr()).abcdsr[0] }
                }
            }

            /// Opaque PeripheralSelectRegister2 (ABCDSR2)
            pub struct PeripheralSelectRegister2 {
                _0: (),
            }

            impl PeripheralSelectRegister2 {
                pub(crate) fn abcdsr2(&mut self) -> &$gpiox::ABCDSR {
                    unsafe { &(*$GPIOX::ptr()).abcdsr[0] }
                }
            }

            /// Opaque InterruptDisableRegister (PIO_IDR)
            pub struct InterruptDisableRegister {
                _0: (),
            }

            impl InterruptDisableRegister {
                pub(crate) fn idr(&mut self) -> &$gpiox::IDR {
                    unsafe { &(*$GPIOX::ptr()).idr }
                }
            }
            
            /// Opaque MultiDriverEnableRegister (PIO_MDER)
            pub struct MultiDriverEnableRegister {
                _0: (),
            }

            impl MultiDriverEnableRegister {
                pub(crate) fn mder(&mut self) -> &$gpiox::MDER {
                    unsafe { &(*$GPIOX::ptr()).mder }
                }
            }

            /// Opaque OutputEnableRegister (PIO_OER)
            pub struct OutputEnableRegister {
                _0: (),
            }

            impl OutputEnableRegister {
                pub(crate) fn oer(&mut self) -> &$gpiox::OER {
                    unsafe { &(*$GPIOX::ptr()).oer }
                }
            }

            /// Opaque PIOEnableRegister (PIO_PER)
            pub struct PIOEnableRegister {
                _0: (),
            }

            impl PIOEnableRegister {
                pub(crate) fn per(&mut self) -> &$gpiox::PER {
                    unsafe { &(*$GPIOX::ptr()).per }
                }
            }
            
            // /// Opaque SODR (Set Output Data Register) register
            // pub struct SODR {
            //     _0: (),
            // }

            // impl SODR {
            //     pub(crate) fn sodr(&mut self) -> &$gpioy::SODR {
            //         unsafe { &(*$GPIOX::ptr()).sodr }
            //     }
            // }

            // /// Opaque CODR (Clear Output Data Register) register
            // pub struct CODR {
            //     _0: (),
            // }

            // impl CODR {
            //     pub(crate) fn codr(&mut self) -> &$gpioy::CODR {
            //         unsafe { &(*$GPIOX::ptr()).codr }
            //     }
            // }

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
                        abcdsr1: &mut PeripheralSelectRegister1,
                        abcdsr2: &mut PeripheralSelectRegister2,
                    ) -> $PXi<PFA> {
                        abcdsr1.abcdsr1().modify(|r, w| unsafe { w.bits(r.bits() & !(1 << $i)) });
                        abcdsr2.abcdsr2().modify(|r, w| unsafe { w.bits(r.bits() & !(1 << $i)) });

                        $PXi { _mode: PhantomData }
                    }

                    // BUGBUG: Rest of peripheral functions here.

                    /// Configures the pin to operate as a floating input pin
                    pub fn into_floating_input(
                        self,
                        pudr: &mut PullUpDisableRegister,
                        ppddr: &mut PadPulldownDisableRegister,
                    ) -> $PXi<Input<Floating>> {
                        pudr.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
                        ppddr.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled down input pin
                    pub fn into_pull_down_input(
                        self,
                        pudr: &mut PullUpDisableRegister,
                        ppder: &mut PadPulldownEnableRegister,
                    ) -> $PXi<Input<PullDown>> {
                        pudr.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) });  // disable pull-up (this must happen first when enabling pull-down resistors)
                        ppder.ppder().write_with_zero(|w| unsafe { w.bits(1 << $i) });  // enable pull-down

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as a pulled up input pin
                    pub fn into_pull_up_input(
                        self,
                        ppddr: &mut PadPulldownDisableRegister,
                        puer: &mut PullUpEnableRegister,
                    ) -> $PXi<Input<PullUp>> {
                        ppddr.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
                        puer.puer().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        $PXi { _mode: PhantomData }
                    }

                    /// Configures the pin to operate as an open drain output pin
                    pub fn into_open_drain_output(
                        self,
                        idr: &mut InterruptDisableRegister,
                        mder: &mut MultiDriverEnableRegister,
                        oer: &mut OutputEnableRegister,
                        per: &mut PIOEnableRegister,
                    ) -> $PXi<Output<OpenDrain>> {
                        // Disable interrupts for pin
                        idr.idr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Enable open-drain/multi-drive
                        mder.mder().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Enable output mode
                        oer.oer().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        // Enable pio mode (disables peripheral control of pin)
                        per.per().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                        $PXi { _mode: PhantomData }
                    }

                    // Configures the pin to operate as an push pull output pin
                    // pub fn into_push_pull_output(
                    //     self,
                    //     moder: &mut MODER,
                    //     otyper: &mut OTYPER,
                    // ) -> $PXi<Output<PushPull>> {
                    //     let offset = 2 * $i;

                    //     // general purpose output mode
                    //     let mode = 0b01;
                    //     moder.moder().modify(|r, w| unsafe {
                    //         w.bits((r.bits() & !(0b11 << offset)) | (mode << offset))
                    //     });

                    //     // push pull output
                    //     otyper
                    //         .otyper()
                    //         .modify(|r, w| unsafe { w.bits(r.bits() & !(0b1 << $i)) });

                    //     $PXi { _mode: PhantomData }
                    // }
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

                // impl<MODE> OutputPin for $PXi<Output<MODE>> {
                //     fn set_high(&mut self) {
                //         // NOTE(unsafe) atomic write to a stateless register
                //         unsafe { (*$GPIOX::ptr()).sodr.write(|w| w.bits(1 << $i)) }
                //     }

                //     fn set_low(&mut self) {
                //         // NOTE(unsafe) atomic write to a stateless register
                //         unsafe { (*$GPIOX::ptr()).codr.write(|w| w.bits(1 << $i)) }
                //     }
                // }
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

// gpio!(GPIOB, gpiob, gpiob, iopben, iopbrst, PBx, [
//     PB0: (pb0, 0, Input<Floating>, AFRL),
//     PB1: (pb1, 1, Input<Floating>, AFRL),
//     PB2: (pb2, 2, Input<Floating>, AFRL),
//     // TODO these are configured as JTAG pins
//     // PB3: (3, Input<Floating>),
//     // PB4: (4, Input<Floating>),
//     PB5: (pb5, 5, Input<Floating>, AFRL),
//     PB6: (pb6, 6, Input<Floating>, AFRL),
//     PB7: (pb7, 7, Input<Floating>, AFRL),
//     PB8: (pb8, 8, Input<Floating>, AFRH),
//     PB9: (pb9, 9, Input<Floating>, AFRH),
//     PB10: (pb10, 10, Input<Floating>, AFRH),
//     PB11: (pb11, 11, Input<Floating>, AFRH),
//     PB12: (pb12, 12, Input<Floating>, AFRH),
//     PB13: (pb13, 13, Input<Floating>, AFRH),
//     PB14: (pb14, 14, Input<Floating>, AFRH),
//     PB15: (pb15, 15, Input<Floating>, AFRH),
// ]);

// gpio!(GPIOC, gpioc, gpioc, iopcen, iopcrst, PCx, [
//     PC0: (pc0, 0, Input<Floating>, AFRL),
//     PC1: (pc1, 1, Input<Floating>, AFRL),
//     PC2: (pc2, 2, Input<Floating>, AFRL),
//     PC3: (pc3, 3, Input<Floating>, AFRL),
//     PC4: (pc4, 4, Input<Floating>, AFRL),
//     PC5: (pc5, 5, Input<Floating>, AFRL),
//     PC6: (pc6, 6, Input<Floating>, AFRL),
//     PC7: (pc7, 7, Input<Floating>, AFRL),
//     PC8: (pc8, 8, Input<Floating>, AFRH),
//     PC9: (pc9, 9, Input<Floating>, AFRH),
//     PC10: (pc10, 10, Input<Floating>, AFRH),
//     PC11: (pc11, 11, Input<Floating>, AFRH),
//     PC12: (pc12, 12, Input<Floating>, AFRH),
//     PC13: (pc13, 13, Input<Floating>, AFRH),
//     PC14: (pc14, 14, Input<Floating>, AFRH),
//     PC15: (pc15, 15, Input<Floating>, AFRH),
// ]);

// gpio!(GPIOD, gpiod, gpioc, iopden, iopdrst, PDx, [
//     PD0: (pd0, 0, Input<Floating>, AFRL),
//     PD1: (pd1, 1, Input<Floating>, AFRL),
//     PD2: (pd2, 2, Input<Floating>, AFRL),
//     PD3: (pd3, 3, Input<Floating>, AFRL),
//     PD4: (pd4, 4, Input<Floating>, AFRL),
//     PD5: (pd5, 5, Input<Floating>, AFRL),
//     PD6: (pd6, 6, Input<Floating>, AFRL),
//     PD7: (pd7, 7, Input<Floating>, AFRL),
//     PD8: (pd8, 8, Input<Floating>, AFRH),
//     PD9: (pd9, 9, Input<Floating>, AFRH),
//     PD10: (pd10, 10, Input<Floating>, AFRH),
//     PD11: (pd11, 11, Input<Floating>, AFRH),
//     PD12: (pd12, 12, Input<Floating>, AFRH),
//     PD13: (pd13, 13, Input<Floating>, AFRH),
//     PD14: (pd14, 14, Input<Floating>, AFRH),
//     PD15: (pd15, 15, Input<Floating>, AFRH),
// ]);

// gpio!(GPIOE, gpioe, gpioc, iopeen, ioperst, PEx, [
//     PE0: (pe0, 0, Input<Floating>, AFRL),
//     PE1: (pe1, 1, Input<Floating>, AFRL),
//     PE2: (pe2, 2, Input<Floating>, AFRL),
//     PE3: (pe3, 3, Input<Floating>, AFRL),
//     PE4: (pe4, 4, Input<Floating>, AFRL),
//     PE5: (pe5, 5, Input<Floating>, AFRL),
//     PE6: (pe6, 6, Input<Floating>, AFRL),
//     PE7: (pe7, 7, Input<Floating>, AFRL),
//     PE8: (pe8, 8, Input<Floating>, AFRH),
//     PE9: (pe9, 9, Input<Floating>, AFRH),
//     PE10: (pe10, 10, Input<Floating>, AFRH),
//     PE11: (pe11, 11, Input<Floating>, AFRH),
//     PE12: (pe12, 12, Input<Floating>, AFRH),
//     PE13: (pe13, 13, Input<Floating>, AFRH),
//     PE14: (pe14, 14, Input<Floating>, AFRH),
//     PE15: (pe15, 15, Input<Floating>, AFRH),
// ]);

// gpio!(GPIOF, gpiof, gpioc, iopfen, iopfrst, PFx, [
//     PF0: (pf0, 0, Input<Floating>, AFRL),
//     PF1: (pf1, 1, Input<Floating>, AFRL),
//     PF2: (pf2, 2, Input<Floating>, AFRL),
//     PF4: (pf3, 4, Input<Floating>, AFRL),
//     PF6: (pf6, 6, Input<Floating>, AFRL),
//     PF9: (pf9, 9, Input<Floating>, AFRH),
//     PF10: (pf10, 10, Input<Floating>, AFRH),
// ]);
