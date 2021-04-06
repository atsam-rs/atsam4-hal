//! General Purpose Input / Output
use {
    core::marker::PhantomData,
    hal::digital::v2::{InputPin, OutputPin},
};

#[cfg(feature = "atsam4e")]
use {
    crate::clock::{Enabled, PioAClock, PioBClock, PioDClock},
    crate::pac::{pioa, piob, piod, PIOA, PIOB, PIOD},
};

#[cfg(feature = "atsam4e_e")]
use {
    crate::clock::{PioCClock, PioEClock},
    crate::pac::{pioc, pioe, PIOC, PIOE},
};

#[cfg(feature = "atsam4s")]
use {
    crate::clock::{Enabled, PioAClock, PioBClock},
    crate::pac::{pioa, piob, PIOA, PIOB},
};

#[cfg(feature = "atsam4s_c")]
use {
    crate::clock::PioCClock,
    crate::pac::{pioc, PIOC},
};

/// The GpioExt trait allows splitting the PORT hardware into
/// its constituent pin parts.
pub trait GpioExt {
    type Parts;

    /// Consume and split the device into its constitent parts
    fn split(self) -> Self::Parts;
}
pub struct Ports {
    pioa: PhantomData<(PIOA, PioAClock<Enabled>)>,
    piob: PhantomData<(PIOB, PioBClock<Enabled>)>,
    #[cfg(any(feature = "atsam4s_c", feature = "atsam4e_e"))]
    pioc: PhantomData<(PIOC, PioCClock<Enabled>)>,
    #[cfg(feature = "atsam4e")]
    piod: PhantomData<(PIOD, PioDClock<Enabled>)>,
    #[cfg(feature = "atsam4e_e")]
    pioe: PhantomData<(PIOE, PioEClock<Enabled>)>,
}

impl Ports {
    pub fn new(
        _pioa: (PIOA, PioAClock<Enabled>),
        _piob: (PIOB, PioBClock<Enabled>),
        #[cfg(any(feature = "atsam4s_c", feature = "atsam4e_e"))] _pioc: (PIOC, PioCClock<Enabled>),
        #[cfg(feature = "atsam4e")] _piod: (PIOD, PioDClock<Enabled>),
        #[cfg(feature = "atsam4e_e")] _pioe: (PIOE, PioEClock<Enabled>),
    ) -> Self {
        // The above arguments are consumed here...never to be seen again.
        Ports {
            pioa: PhantomData,
            piob: PhantomData,
            #[cfg(any(feature = "atsam4s_c", feature = "atsam4e_e"))]
            pioc: PhantomData,
            #[cfg(feature = "atsam4e")]
            piod: PhantomData,
            #[cfg(feature = "atsam4e_e")]
            pioe: PhantomData,
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

/// Peripheral Function A
pub struct PfA;
/// Peripheral Function B
pub struct PfB;
/// Peripheral Function C
pub struct PfC;
/// Peripheral Function D
pub struct PfD;

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

macro_rules! pins {
    ([
        $($PinTypeA:ident: ($pin_identA:ident, $pin_noA:expr),)+
    ],[
        $($PinTypeB:ident: ($pin_identB:ident, $pin_noB:expr),)+
    ],[
        $($PinTypeC:ident: ($pin_identC:ident, $pin_noC:expr),)+
    ],[
        $($PinTypeD:ident: ($pin_identD:ident, $pin_noD:expr),)+
    ],[
        $($PinTypeE:ident: ($pin_identE:ident, $pin_noE:expr),)+
    ]) => {
        /// Holds the GPIO broken out pin instances (consumes the Ports object)
        pub struct Pins {
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
                #[cfg(any(feature = "atsam4s_c", feature = "atsam4e_e"))]
                pub $pin_identC: $PinTypeC<Input<Floating>>,
            )+
            $(
                /// Pin $pin_identD
                #[cfg(feature = "atsam4e")]
                pub $pin_identD: $PinTypeD<Input<Floating>>,
            )+
            $(
                /// Pin $pin_identE
                #[cfg(feature = "atsam4e_e")]
                pub $pin_identE: $PinTypeE<Input<Floating>>,
            )+
        }

        impl GpioExt for Ports {
            type Parts = Pins;

            /// Split the PORT peripheral into discrete pins
            fn split(self) -> Pins {
                Pins {
                    $(
                        $pin_identA: $PinTypeA { _mode: PhantomData },
                    )+
                    $(
                        $pin_identB: $PinTypeB { _mode: PhantomData },
                    )+
                    $(
                        #[cfg(any(feature = "atsam4s_c", feature = "atsam4e_e"))]
                        $pin_identC: $PinTypeC { _mode: PhantomData },
                    )+
                    $(
                        #[cfg(feature = "atsam4e")]
                        $pin_identD: $PinTypeD { _mode: PhantomData },
                    )+
                    $(
                        #[cfg(feature = "atsam4e_e")]
                        $pin_identE: $PinTypeE { _mode: PhantomData },
                    )+
                }
            }
        }

        $(
            pin!($PinTypeA, $pin_identA, $pin_noA, PIOA, pioa);
        )+
        $(
            pin!($PinTypeB, $pin_identB, $pin_noB, PIOB, piob);
        )+
        $(
            #[cfg(any(feature = "atsam4s_c", feature = "atsam4e_e"))]
            pin!($PinTypeC, $pin_identC, $pin_noC, PIOC, pioc);
        )+
        $(
            #[cfg(feature = "atsam4e")]
            pin!($PinTypeD, $pin_identD, $pin_noD, PIOD, piod);
        )+
        $(
            #[cfg(feature = "atsam4e_e")]
            pin!($PinTypeE, $pin_identE, $pin_noE, PIOE, pioe);
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
        pub struct $PinType<MODE> {
            _mode: PhantomData<MODE>,
        }

        impl<MODE> $PinType<MODE> {
            pub(crate) fn puer(&mut self) -> &$pio::PUER {
                unsafe { &(*$PIO::ptr()).puer }
            }

            pub(crate) fn pudr(&mut self) -> &$pio::PUDR {
                unsafe { &(*$PIO::ptr()).pudr }
            }

            pub(crate) fn _ier(&mut self) -> &$pio::IER {
                unsafe { &(*$PIO::ptr()).ier }
            }

            pub(crate) fn idr(&mut self) -> &$pio::IDR {
                unsafe { &(*$PIO::ptr()).idr }
            }

            pub(crate) fn ppder(&mut self) -> &$pio::PPDER {
                unsafe { &(*$PIO::ptr()).ppder }
            }

            pub(crate) fn ppddr(&mut self) -> &$pio::PPDDR {
                unsafe { &(*$PIO::ptr()).ppddr }
            }

            pub(crate) fn abcdsr1(&mut self) -> &$pio::ABCDSR {
                unsafe { &(*$PIO::ptr()).abcdsr[0] }
            }

            pub(crate) fn abcdsr2(&mut self) -> &$pio::ABCDSR {
                unsafe { &(*$PIO::ptr()).abcdsr[1] }
            }

            pub(crate) fn mder(&mut self) -> &$pio::MDER {
                unsafe { &(*$PIO::ptr()).mder }
            }

            pub(crate) fn mddr(&mut self) -> &$pio::MDDR {
                unsafe { &(*$PIO::ptr()).mddr }
            }

            pub(crate) fn oer(&mut self) -> &$pio::OER {
                unsafe { &(*$PIO::ptr()).oer }
            }

            pub(crate) fn odr(&mut self) -> &$pio::ODR {
                unsafe { &(*$PIO::ptr()).odr }
            }

            pub(crate) fn per(&mut self) -> &$pio::PER {
                unsafe { &(*$PIO::ptr()).per }
            }

            pub(crate) fn pdr(&mut self) -> &$pio::PDR {
                unsafe { &(*$PIO::ptr()).pdr }
            }

            pub(crate) fn sodr(&mut self) -> &$pio::SODR {
                unsafe { &(*$PIO::ptr()).sodr }
            }

            pub(crate) fn codr(&mut self) -> &$pio::CODR {
                unsafe { &(*$PIO::ptr()).codr }
            }

            pub(crate) fn ifscdr(&mut self) -> &$pio::IFSCDR {
                unsafe { &(*$PIO::ptr()).ifscdr }
            }

            fn enable_pin(&mut self) {
                self.per().write_with_zero(|w| unsafe { w.bits(1 << $i) });
            }

            fn disable_pin(&mut self) {
                self.pdr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
            }

            fn _enable_pin_interrupt(&mut self) {
                self._ier().write_with_zero(|w| unsafe { w.bits(1 << $i) });
            }

            fn disable_pin_interrupt(&mut self) {
                self.idr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
            }

            fn prepare_pin_for_function_use(&mut self) {
                self.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable Pullup
                self.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable Pulldown
                self.mddr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable Multi-drive (open drain)
                self.ifscdr()
                    .write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable Glitch filter (Debounce)
            }

            fn prepare_pin_for_input_use(&mut self) {
                self.disable_pin_interrupt(); // Disable interrupt
                self.mddr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable open-drain/multi-drive
                self.odr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable output mode
            }

            pub fn into_peripheral_function_a(mut self) -> $PinType<PfA> {
                self.prepare_pin_for_function_use();
                self.abcdsr1()
                    .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << $i)) });
                self.abcdsr2()
                    .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << $i)) });
                self.disable_pin();

                $PinType { _mode: PhantomData }
            }

            pub fn into_peripheral_function_b(mut self) -> $PinType<PfB> {
                self.prepare_pin_for_function_use();
                self.abcdsr1()
                    .modify(|r, w| unsafe { w.bits(r.bits() | (1 << $i)) }); // Set up peripheral function
                self.abcdsr2()
                    .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << $i)) });
                self.disable_pin();

                $PinType { _mode: PhantomData }
            }

            pub fn into_peripheral_function_c(mut self) -> $PinType<PfC> {
                self.prepare_pin_for_function_use();
                self.abcdsr1()
                    .modify(|r, w| unsafe { w.bits(r.bits() & !(1 << $i)) }); // Set up peripheral function
                self.abcdsr2()
                    .modify(|r, w| unsafe { w.bits(r.bits() | (1 << $i)) });
                self.disable_pin();

                $PinType { _mode: PhantomData }
            }

            pub fn into_peripheral_function_d(mut self) -> $PinType<PfD> {
                self.prepare_pin_for_function_use();
                self.abcdsr1()
                    .modify(|r, w| unsafe { w.bits(r.bits() | (1 << $i)) }); // Set up peripheral function
                self.abcdsr2()
                    .modify(|r, w| unsafe { w.bits(r.bits() | (1 << $i)) });
                self.disable_pin();

                $PinType { _mode: PhantomData }
            }

            pub fn into_floating_input(mut self) -> $PinType<Input<Floating>> {
                self.prepare_pin_for_input_use();
                self.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable pull-up
                self.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable pull-down
                self.enable_pin();

                $PinType { _mode: PhantomData }
            }

            pub fn into_pull_down_input(mut self) -> $PinType<Input<PullDown>> {
                self.prepare_pin_for_input_use();
                self.pudr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable pull-up (this must happen first when enabling pull-down resistors)
                self.ppder().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Enable pull-down
                self.enable_pin();

                $PinType { _mode: PhantomData }
            }

            pub fn into_pull_up_input(mut self) -> $PinType<Input<PullUp>> {
                self.prepare_pin_for_input_use();
                self.ppddr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable pull-down
                self.puer().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Enable pull-up
                self.enable_pin();

                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to operate as an open drain output
            pub fn into_open_drain_output(mut self) -> $PinType<Output<OpenDrain>> {
                self.disable_pin_interrupt();
                self.mder().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Enable open-drain/multi-drive
                self.oer().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Enable output mode
                self.enable_pin(); // Enable pio mode (disables peripheral control of pin)

                $PinType { _mode: PhantomData }
            }

            /// Configures the pin to operate as a push-pull output
            pub fn into_push_pull_output(mut self) -> $PinType<Output<PushPull>> {
                self.disable_pin_interrupt();
                self.mddr().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Disable open-drain/multi-drive
                self.oer().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Enable output mode
                self.per().write_with_zero(|w| unsafe { w.bits(1 << $i) }); // Enable pio mode (disables peripheral control of pin)

                $PinType { _mode: PhantomData }
            }
        }

        impl<MODE> InputPin for $PinType<Input<MODE>> {
            type Error = ();

            fn is_high(&self) -> Result<bool, Self::Error> {
                Ok(false)
                //                Ok(unsafe { (((*PORT::ptr()).$in.read().bits()) & (1 << $pin_no)) != 0 })
            }

            fn is_low(&self) -> Result<bool, Self::Error> {
                Ok(false)
                //                Ok(unsafe { (((*PORT::ptr()).$in.read().bits()) & (1 << $pin_no)) == 0 })
            }
        }

        impl<MODE> OutputPin for $PinType<Output<MODE>> {
            type Error = ();

            fn set_high(&mut self) -> Result<(), Self::Error> {
                self.sodr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
                Ok(())
            }

            fn set_low(&mut self) -> Result<(), Self::Error> {
                self.codr().write_with_zero(|w| unsafe { w.bits(1 << $i) });
                Ok(())
            }
        }
    };
}

pins!([
    Pa0: (pa0, 0),
    Pa1: (pa1, 1),
    Pa2: (pa2, 2),
    Pa3: (pa3, 3),
    Pa4: (pa4, 4),
    Pa5: (pa5, 5),
    Pa6: (pa6, 6),
    Pa7: (pa7, 7),
    Pa8: (pa8, 8),
    Pa9: (pa9, 9),
    Pa10: (pa10, 10),
    Pa11: (pa11, 11),
    Pa12: (pa12, 12),
    Pa13: (pa13, 13),
    Pa14: (pa14, 14),
    Pa15: (pa15, 15),
    Pa16: (pa16, 16),
    Pa17: (pa17, 17),
    Pa18: (pa18, 18),
    Pa19: (pa19, 19),
    Pa20: (pa20, 20),
    Pa21: (pa21, 21),
    Pa22: (pa22, 22),
    Pa23: (pa23, 23),
    Pa24: (pa24, 24),
    Pa25: (pa25, 25),
    Pa26: (pa26, 26),
    Pa27: (pa27, 27),
    Pa28: (pa28, 28),
    Pa29: (pa29, 29),
    Pa30: (pa30, 30),
    Pa31: (pa31, 31),
],[
    Pb0: (pb0, 0),
    Pb1: (pb1, 1),
    Pb2: (pb2, 2),
    Pb3: (pb3, 3),
    Pb4: (pb4, 4),
    Pb5: (pb5, 5),
    Pb6: (pb6, 6),
    Pb7: (pb7, 7),
    Pb8: (pb8, 8),
    Pb9: (pb9, 9),
    Pb10: (pb10, 10),
    Pb11: (pb11, 11),
    Pb12: (pb12, 12),
    Pb13: (pb13, 13),
    Pb14: (pb14, 14),

    // PB15-31 do not exist.
],
[
    Pc0: (pc0, 0),
    Pc1: (pc1, 1),
    Pc2: (pc2, 2),
    Pc3: (pc3, 3),
    Pc4: (pc4, 4),
    Pc5: (pc5, 5),
    Pc6: (pc6, 6),
    Pc7: (pc7, 7),
    Pc8: (pc8, 8),
    Pc9: (pc9, 9),
    Pc10: (pc10, 10),
    Pc11: (pc11, 11),
    Pc12: (pc12, 12),
    Pc13: (pc13, 13),
    Pc14: (pc14, 14),
    Pc15: (pc15, 15),
    Pc16: (pc16, 16),
    Pc17: (pc17, 17),
    Pc18: (pc18, 18),
    Pc19: (pc19, 19),
    Pc20: (pc20, 20),
    Pc21: (pc21, 21),
    Pc22: (pc22, 22),
    Pc23: (pc23, 23),
    Pc24: (pc24, 24),
    Pc25: (pc25, 25),
    Pc26: (pc26, 26),
    Pc27: (pc27, 27),
    Pc28: (pc28, 28),
    Pc29: (pc29, 29),
    Pc30: (pc30, 30),
    Pc31: (pc31, 31),
],
[
    Pd0: (pd0, 0),
    Pd1: (pd1, 1),
    Pd2: (pd2, 2),
    Pd3: (pd3, 3),
    Pd4: (pd4, 4),
    Pd5: (pd5, 5),
    Pd6: (pd6, 6),
    Pd7: (pd7, 7),
    Pd8: (pd8, 8),
    Pd9: (pd9, 9),
    Pd10: (pd10, 10),
    Pd11: (pd11, 11),
    Pd12: (pd12, 12),
    Pd13: (pd13, 13),
    Pd14: (pd14, 14),
    Pd15: (pd15, 15),
    Pd16: (pd16, 16),
    Pd17: (pd17, 17),
    Pd18: (pd18, 18),
    Pd19: (pd19, 19),
    Pd20: (pd20, 20),
    Pd21: (pd21, 21),
    Pd22: (pd22, 22),
    Pd23: (pd23, 23),
    Pd24: (pd24, 24),
    Pd25: (pd25, 25),
    Pd26: (pd26, 26),
    Pd27: (pd27, 27),
    Pd28: (pd28, 28),
    Pd29: (pd29, 29),
    Pd30: (pd30, 30),
    Pd31: (pd31, 31),
],
[
    Pe0: (pe0, 0),
    Pe1: (pe1, 1),
    Pe2: (pe2, 2),
    Pe3: (pe3, 3),
    Pe4: (pe4, 4),
    Pe5: (pe5, 5),

    // Pe6-31 do not exist.
]);

#[macro_export]
macro_rules! define_pin_map {
    ($(#[$topattr:meta])* struct $Type:ident,
     $( $(#[$attr:meta])* pin $name:ident = $pin_ident:ident<$pin_type:ty, $into_method:ident>),+ , ) => {

        paste! {
            $(#[$topattr])*
            pub struct $Type {
                $(
                    $(#[$attr])*
                    pub $name: [<P $pin_ident>]<$pin_type>
                ),+
            }
        }

        impl $Type {
            /// Returns the pins for the device
            paste! {
                pub fn new(ports: Ports) -> Self {
                    let pins = ports.split();
                    // Create local pins with the correct type so we can put them into the
                    // pin structure below.
                    $(
                        let [<new_pin $pin_ident>] = pins.[<p $pin_ident>].$into_method();
                    )+
                    $Type {
                        $(
                            $name: [<new_pin $pin_ident>]
                        ),+
                    }
                }
            }
        }
    }
}
