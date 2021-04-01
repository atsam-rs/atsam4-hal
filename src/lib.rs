//! HAL for the ATSAM4 series of microcontrollers
//!
//! This is an implementation of the [`embedded-hal`] traits for the ATSAM4 microcontrollers
//!
//! [`embedded-hal`]: https://github.com/japaric/embedded-hal
//!
//! # Requirements
//!
//! This crate requires `arm-none-eabi-gcc` to be installed and available in `$PATH` to build.
//!
//! # Usage
//!
//! To build applications (binary crates) using this crate follow the [cortex-m-quickstart]
//! instructions and add this crate as a dependency in step number 5 and make sure you enable the
//! "rt" Cargo feature of this crate.
//!
//! [cortex-m-quickstart]: https://docs.rs/cortex-m-quickstart/~0.3
//!

//#![deny(missing_docs)]
//#![deny(warnings)]
#![no_std]
// Needed to quiet names such as NCS1 and UART0Clock (which now throw linting errors)
// These errors seem to have been removed in nightly, so I suspect they may not stay.
#![allow(clippy::upper_case_acronyms)]

#[macro_use]
extern crate lazy_static;

pub extern crate embedded_hal as hal;
pub use hal::digital::v2::*;

#[cfg(feature = "net")]
extern crate smoltcp;

#[cfg(feature = "atsam4e16e")]
pub use atsam4e16e_pac as pac;

#[cfg(feature = "atsam4s4b")]
pub use atsam4s4b_pac as pac;

#[cfg(feature = "atsam4s8b")]
pub use atsam4s8b_pac as pac;

#[cfg(feature = "atsam4sd32c")]
pub use atsam4sd32c_pac as pac;

pub use eui48::Identifier as MacAddress;

use core::mem;

pub mod clock;
pub mod delay;
pub mod gpio;
pub mod prelude;
pub mod serial;
pub mod static_memory_controller;
pub mod time;
pub mod watchdog;

#[cfg(all(feature = "atsam4e16e", feature = "unstable"))]
#[allow(dead_code)] // TODO: REMOVE WHEN STABLE
pub mod ethernet_controller;

/// Borrows a peripheral without checking if it has already been taken
unsafe trait BorrowUnchecked {
    fn borrow_unchecked<T>(f: impl FnOnce(&mut Self) -> T) -> T;
}

macro_rules! borrow_unchecked {
    ($($peripheral:ident),*) => {
        $(
            unsafe impl BorrowUnchecked for pac::$peripheral {
                fn borrow_unchecked<T>(f: impl FnOnce(&mut Self) -> T) -> T {
                    let mut p = unsafe { mem::transmute(()) };
                    f(&mut p)
                }
            }
        )*
    }
}

borrow_unchecked!(PMC);
