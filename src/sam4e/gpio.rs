//! General Purpose Input / Output
use {
    atsam4e16e_pac::{
        PIOA, pioa,
        PIOB, piob,
        PIOC, pioc,
        PIOD, piod,
    },
    core::marker::PhantomData,
    embedded_hal::digital::v2::OutputPin,
};

/// The GpioExt trait allows splitting the PORT hardware into
/// its constituent pin parts.
pub trait GpioExt {
    type Parts;

    /// Consume and split the device into its constitent parts
    fn split(self) -> Self::Parts;
}

pub struct PortAggregate {
}

impl PortAggregate {
    pub fn new(_pioa: PIOA, _piob: PIOB, _pioc: PIOC, _piod: PIOD) -> Self {
        // The above arguments are consumed here...never to be seen again.
        PortAggregate {
        }
    }
}

/// Represents a pin configured for input.
/// The MODE type is typically one of `Floating`, `PullDown` or
/// `PullUp`.
pub struct Input<MODE> {
    _mode: PhantomData<MODE>,
}

/// Represents a pin configured for output.
/// The MODE type is typically one of `PushPull`, or
/// `OpenDrain`.
pub struct Output<MODE> {
    _mode: PhantomData<MODE>,
}

/// Floating Input
pub struct Floating;
/// Pulled down Input
pub struct PullDown;
/// Pulled up Input
pub struct PullUp;

/// Totem Pole aka Push-Pull
pub struct PushPull;
/// Open drain output
pub struct OpenDrain;

macro_rules! create_port_peripheral_struct {
    ($TypeName:ident, $GPIO:ident, $gpio:ident) => {
        pub(crate) struct $TypeName {
            _0: (),
        }

        impl $TypeName {
            pub(crate) fn puer(&mut self) -> &$gpio::PUER {
                unsafe { &(*$GPIO::ptr()).puer }
            }
    
            pub(crate) fn pudr(&mut self) -> &$gpio::PUDR {
                unsafe { &(*$GPIO::ptr()).pudr }
            }
    
            pub(crate) fn ppder(&mut self) -> &$gpio::PPDER {
                unsafe { &(*$GPIO::ptr()).ppder }
            }
    
            pub(crate) fn ppddr(&mut self) -> &$gpio::PPDDR {
                unsafe { &(*$GPIO::ptr()).ppddr }
            }
    
            pub(crate) fn _abcdsr1(&mut self) -> &$gpio::ABCDSR {
                unsafe { &(*$GPIO::ptr()).abcdsr[0] }
            }
    
            pub(crate) fn _abcdsr2(&mut self) -> &$gpio::ABCDSR {
                unsafe { &(*$GPIO::ptr()).abcdsr[1] }
            }
    
            pub(crate) fn idr(&mut self) -> &$gpio::IDR {
                unsafe { &(*$GPIO::ptr()).idr }
            }
    
            pub(crate) fn mder(&mut self) -> &$gpio::MDER {
                unsafe { &(*$GPIO::ptr()).mder }
            }
    
            pub(crate) fn mddr(&mut self) -> &$gpio::MDDR {
                unsafe { &(*$GPIO::ptr()).mddr }
            }
    
            pub(crate) fn oer(&mut self) -> &$gpio::OER {
                unsafe { &(*$GPIO::ptr()).oer }
            }
    
            pub(crate) fn per(&mut self) -> &$gpio::PER {
                unsafe { &(*$GPIO::ptr()).per }
            }      

            pub(crate) fn _sodr(&mut self) -> &$gpio::SODR {
                unsafe { &(*$GPIO::ptr()).sodr }
            }

            pub(crate) fn _codr(&mut self) -> &$gpio::CODR {
                unsafe { &(*$GPIO::ptr()).codr }
            }
        }      
    }
}

create_port_peripheral_struct!(PortA, PIOA, pioa);
create_port_peripheral_struct!(PortB, PIOB, piob);
create_port_peripheral_struct!(PortC, PIOC, pioc);
create_port_peripheral_struct!(PortD, PIOD, piod);

/// Opaque port reference
pub struct Port {
    porta: PortA,
    portb: PortB,
    portc: PortC,
    portd: PortD,
}

impl Port {
}

macro_rules! port {
    ([
        $($PinTypeA:ident: ($pin_identA:ident, $pin_noA:expr),)+
    ],[
        $($PinTypeB:ident: ($pin_identB:ident, $pin_noB:expr),)+
    ],[
        $($PinTypeC:ident: ($pin_identC:ident, $pin_noC:expr),)+
    ],[
        $($PinTypeD:ident: ($pin_identD:ident, $pin_noD:expr),)+
    ]) => {
        /// Holds the GPIO Port peripheral and broken out pin instances
        pub struct Parts {
            /// Opaque port reference
            pub port: Port,
        
            $(
                /// Pin $pin_identA
                pub $pin_identA: $PinTypeA<Input<Floating>>,
            )+
            $(
                /// Pin $pin_identB
                pub $pin_identB: $PinTypeB<Input<Floating>>,
            )+
            $(
                /// Pin $pin_identC
                pub $pin_identC: $PinTypeC<Input<Floating>>,
            )+
            $(
                /// Pin $pin_identD
                pub $pin_identD: $PinTypeD<Input<Floating>>,
            )+
        }
        
        impl GpioExt for PortAggregate {
            type Parts = Parts;
        
            /// Split the PORT peripheral into discrete pins
            fn split(self) -> Parts {
                Parts {
                    port: Port {
                        porta: PortA { _0: () },
                        portb: PortB { _0: () },
                        portc: PortC { _0: () },
                        portd: PortD { _0: () },
                    },
                    $(
                        $pin_identA: $PinTypeA { _mode: PhantomData },
                    )+
                    $(
                        $pin_identB: $PinTypeB { _mode: PhantomData },
                    )+
                    $(
                        $pin_identC: $PinTypeC { _mode: PhantomData },
                    )+
                    $(
                        $pin_identD: $PinTypeD { _mode: PhantomData },
                    )+
                }
            }
        }
        
        $(
            pin!($PinTypeA, $pin_identA, $pin_noA, PIOA, porta);
        )+
        $(
            pin!($PinTypeB, $pin_identB, $pin_noB, PIOB, portb);
        )+
        $(
            pin!($PinTypeC, $pin_identC, $pin_noC, PIOC, portc);
        )+
        $(
            pin!($PinTypeD, $pin_identD, $pin_noD, PIOD, portd);
        )+    
    };
}    

macro_rules! pin {
    (
        $PinType:ident,
        $pin_ident:ident,
        $i:expr,
        $PIO:ident,
        $pio:ident
    ) => {
        // // Helper for pmux peripheral function configuration
        // macro_rules! function {
        //     ($FuncType:ty, $func_ident:ident, $variant:ident) => {
        //         impl<MODE> $PinType<MODE> {
        //             /// Configures the pin to operate with a peripheral
        //             pub fn $func_ident(
        //                 self,
        //                 port: &mut Port
        //             ) -> $PinType<$FuncType> {
        //                 port.$pinmux()[$pin_no >> 1].modify(|_, w| {
        //                     if $pin_no & 1 == 1 {
        //                         // Odd-numbered pin
        //                         w.pmuxo().$variant()
        //                     } else {
        //                         // Even-numbered pin
        //                         w.pmuxe().$variant()
        //                     }
        //                 });
        //                 port.$pincfg()[$pin_no].modify(|_, bits| {
        //                     bits.pmuxen().set_bit()
        //                 });

        //                 $PinType { _mode: PhantomData }
        //             }
        //         }

        //         impl<MODE> IntoFunction<$PinType<$FuncType>> for $PinType<MODE> {
        //             fn into_function(self, port: &mut Port) -> $PinType<$FuncType> {
        //                 self.$func_ident(port)
        //             }
        //         }
        //     };
        // }

        /// Represents the IO pin with the matching name.
        pub struct $PinType<MODE> {
            _mode: PhantomData<MODE>,
        }

        // function!(PfA, into_function_a, a);
        // function!(PfB, into_function_b, b);
        // function!(PfC, into_function_c, c);
        // function!(PfD, into_function_d, d);
        // function!(PfE, into_function_e, e);
        // function!(PfF, into_function_f, f);
        // function!(PfG, into_function_g, g);
        // function!(PfH, into_function_h, h);

        // #[cfg(any(feature = "samd51", feature = "same54"))]
        // function!(PfI, into_function_i, i);
        // #[cfg(any(feature = "samd51", feature = "same54"))]
        // function!(PfJ, into_function_j, j);
        // #[cfg(any(feature = "samd51", feature = "same54"))]
        // function!(PfK, into_function_k, k);
        // #[cfg(any(feature = "samd51", feature = "same54"))]
        // function!(PfL, into_function_l, l);
        // #[cfg(any(feature = "samd51", feature = "same54"))]
        // function!(PfM, into_function_m, m);
        // #[cfg(any(feature = "samd51", feature = "same54"))]
        // function!(PfN, into_function_n, n);

        impl<MODE> $PinType<MODE> {

            pub fn into_floating_input(self, port: &mut Port) -> $PinType<Input<Floating>> {
                port.$pio.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
                port.$pio.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                $PinType { _mode: PhantomData }
            }

            pub fn into_pull_down_input(self, port: &mut Port) -> $PinType<Input<PullDown>> {
                port.$pio.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) });  // disable pull-up (this must happen first when enabling pull-down resistors)
                port.$pio.ppder().write_with_zero(|w| unsafe { w.bits(1 << $i) });  // enable pull-down

                $PinType { _mode: PhantomData }
            }

            pub fn into_pull_up_input(self, port: &mut Port) -> $PinType<Input<PullUp>> {
                port.$pio.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
                port.$pio.puer().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to operate as an open drain output
            pub fn into_open_drain_output(self, port: &mut Port) -> $PinType<Output<OpenDrain>> {
                // Disable interrupts for pin
                port.$pio.idr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                // Enable open-drain/multi-drive
                port.$pio.mder().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                // Enable output mode
                port.$pio.oer().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                // Enable pio mode (disables peripheral control of pin)
                port.$pio.per().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to operate as a push-pull output
            pub fn into_push_pull_output(self, port: &mut Port) -> $PinType<Output<PushPull>> {
                // Disable interrupts for pin
                port.$pio.idr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                // Disable open-drain/multi-drive
                port.$pio.mddr().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                // Enable output mode
                port.$pio.oer().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                // Enable pio mode (disables peripheral control of pin)
                port.$pio.per().write_with_zero(|w| unsafe { w.bits(1 << $i) });

                $PinType { _mode: PhantomData }
            }
        }

        // impl $PinType<Output<OpenDrain>> {
        //     /// Control state of the internal pull up
        //     pub fn internal_pull_up(&mut self, port: &mut Port, on: bool) {
        //         port.$pincfg()[$pin_no].write(|bits| {
        //             if on {
        //                 bits.pullen().set_bit();
        //             } else {
        //                 bits.pullen().clear_bit();
        //             }
        //             bits
        //         });
        //     }
        // }

        // impl<MODE> $PinType<Output<MODE>> {
        //     /// Toggle the logic level of the pin; if it is currently
        //     /// high, set it low and vice versa.
        //     pub fn toggle(&mut self) {
        //         self.toggle_impl();
        //     }

        //     fn toggle_impl(&mut self) {
        //         unsafe {
        //             (*PORT::ptr()).$outtgl.write(|bits| {
        //                 bits.bits(1 << $pin_no);
        //                 bits
        //             });
        //         }
        //     }
        // }

        // #[cfg(feature = "unproven")]
        // impl<MODE> ToggleableOutputPin for $PinType<Output<MODE>> {
        //     // TODO: switch to ! when it’s stable
        //     type Error = ();

        //     fn toggle(&mut self) -> Result<(), Self::Error> {
        //         self.toggle_impl();

        //         Ok(())
        //     }
        // }

        // #[cfg(feature = "unproven")]
        // impl InputPin for $PinType<Output<ReadableOpenDrain>> {
        //     // TODO: switch to ! when it’s stable
        //     type Error = ();

        //     fn is_high(&self) -> Result<bool, Self::Error> {
        //         Ok(unsafe { (((*PORT::ptr()).$in.read().bits()) & (1 << $pin_no)) != 0 })
        //     }

        //     fn is_low(&self) -> Result<bool, Self::Error> {
        //         Ok(unsafe { (((*PORT::ptr()).$in.read().bits()) & (1 << $pin_no)) == 0 })
        //     }
        // }

        // #[cfg(feature = "unproven")]
        // impl<MODE> InputPin for $PinType<Input<MODE>> {
        //     // TODO: switch to ! when it’s stable
        //     type Error = ();

        //     fn is_high(&self) -> Result<bool, Self::Error> {
        //         Ok(unsafe { (((*PORT::ptr()).$in.read().bits()) & (1 << $pin_no)) != 0 })
        //     }

        //     fn is_low(&self) -> Result<bool, Self::Error> {
        //         Ok(unsafe { (((*PORT::ptr()).$in.read().bits()) & (1 << $pin_no)) == 0 })
        //     }
        // }

        // #[cfg(feature = "unproven")]
        // impl<MODE> StatefulOutputPin for $PinType<Output<MODE>> {
        //     fn is_set_high(&self) -> Result<bool, Self::Error> {
        //         Ok(unsafe { (((*PORT::ptr()).$out.read().bits()) & (1 << $pin_no)) != 0 })
        //     }

        //     fn is_set_low(&self) -> Result<bool, Self::Error> {
        //         Ok(unsafe { (((*PORT::ptr()).$out.read().bits()) & (1 << $pin_no)) == 0 })
        //     }
        // }

        impl<MODE> OutputPin for $PinType<Output<MODE>> {
            type Error = ();

            fn set_high(&mut self) -> Result<(), Self::Error> {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*$PIO::ptr()).sodr.write_with_zero(|w| w.bits(1 << $i)) }
                Ok(())
            }

            fn set_low(&mut self) -> Result<(), Self::Error> {
                // NOTE(unsafe) atomic write to a stateless register
                unsafe { (*$PIO::ptr()).codr.write_with_zero(|w| w.bits(1 << $i)) }
                Ok(())
            }
}
    };
}

port!([
    PA0: (pa0, 0),
    PA1: (pa1, 1),
    PA2: (pa2, 2),
    PA3: (pa3, 3),
    PA4: (pa4, 4),
    PA5: (pa5, 5),
    PA6: (pa6, 6),
    PA7: (pa7, 7),
    PA8: (pa8, 8),
    PA9: (pa9, 9),
    PA10: (pa10, 10),
    PA11: (pa11, 11),
    PA12: (pa12, 12),
    PA13: (pa13, 13),
    PA14: (pa14, 14),
    PA15: (pa15, 15),
    PA16: (pa16, 16),
    PA17: (pa17, 17),
    PA18: (pa18, 18),
    PA19: (pa19, 19),
    PA20: (pa20, 20),
    PA21: (pa21, 21),
    PA22: (pa22, 22),
    PA23: (pa23, 23),
    PA24: (pa24, 24),
    PA25: (pa25, 25),
    PA26: (pa26, 26),
    PA27: (pa27, 27),
    PA28: (pa28, 28),
    PA29: (pa29, 29),
    PA30: (pa30, 30),
    PA31: (pa31, 31),
],[
    PB0: (pb0, 0),
    PB1: (pb1, 1),
    PB2: (pb2, 2),
    PB3: (pb3, 3),
    PB4: (pb4, 4),
    PB5: (pb5, 5),
    PB6: (pb6, 6),
    PB7: (pb7, 7),
    PB8: (pb8, 8),
    PB9: (pb9, 9),
    PB10: (pb10, 10),
    PB11: (pb11, 11),
    PB12: (pb12, 12),
    PB13: (pb13, 13),
    PB14: (pb14, 14),

    // PB15-31 do not exist.

],
[
    PC0: (pc0, 0),
    PC1: (pc1, 1),
    PC2: (pc2, 2),
    PC3: (pc3, 3),
    PC4: (pc4, 4),
    PC5: (pc5, 5),
    PC6: (pc6, 6),
    PC7: (pc7, 7),
    PC10: (pc10, 10),
    PC11: (pc11, 11),
    PC12: (pc12, 12),
    PC13: (pc13, 13),
    PC14: (pc14, 14),
    PC15: (pc15, 15),
    PC16: (pc16, 16),
    PC17: (pc17, 17),
    PC18: (pc18, 18),
    PC19: (pc19, 19),
    PC20: (pc20, 20),
    PC21: (pc21, 21),
    PC22: (pc22, 22),
    PC23: (pc23, 23),
    PC24: (pc24, 24),
    PC25: (pc25, 25),
    PC26: (pc26, 26),
    PC27: (pc27, 27),
    PC28: (pc28, 28),
    PC30: (pc30, 30),
    PC31: (pc31, 31),
],
[
    PD0: (pd0, 0),
    PD1: (pd1, 1),
    PD2: (pd2, 2),
    PD3: (pd3, 3),
    PD4: (pd4, 4),
    PD5: (pd5, 5),
    PD6: (pd6, 6),
    PD7: (pd7, 7),
    PD10: (pd10, 10),
    PD11: (pd11, 11),
    PD12: (pd12, 12),
    PD13: (pd13, 13),
    PD14: (pd14, 14),
    PD15: (pd15, 15),
    PD16: (pd16, 16),
    PD17: (pd17, 17),
    PD18: (pd18, 18),
    PD19: (pd19, 19),
    PD20: (pd20, 20),
    PD21: (pd21, 21),
    PD22: (pd22, 22),
    PD23: (pd23, 23),
    PD24: (pd24, 24),
    PD25: (pd25, 25),
    PD26: (pd26, 26),
    PD27: (pd27, 27),
    PD28: (pd28, 28),
    PD30: (pd30, 30),
    PD31: (pd31, 31),
]);
