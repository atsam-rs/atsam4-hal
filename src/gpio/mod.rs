use core::marker::PhantomData;

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

#[cfg(feature = "atsam4")]
pub mod atsam4;
#[cfg(feature = "atsam4")]
pub use atsam4::*;
