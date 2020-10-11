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

#[macro_use]
extern crate lazy_static;

pub extern crate embedded_hal;
pub use embedded_hal::digital::v2::*;

#[cfg(feature = "net")]
extern crate smoltcp;

#[cfg(feature = "atsam4e16e")]
pub use atsam4e16e_pac as pac;

#[cfg(feature = "atsam4sd32c")]
pub use atsam4sd32c_pac as pac;

pub use eui48::Identifier as MacAddress;

use cortex_m_rt::pre_init;
use core::mem;

pub mod clock;
pub mod delay;
pub mod gpio;
pub mod static_memory_controller;
pub mod serial;
pub mod time;

#[cfg(all(feature = "atsam4e16e", feature = "unstable"))]
#[allow(dead_code)]             // TODO: REMOVE WHEN STABLE
pub mod ethernet_controller;

// peripheral initialization
#[pre_init]
unsafe fn pre_init() {
    // Disable the watchdog timer if requested.
    #[cfg(feature = "disable_watchdog_timer")]
    pac::WDT::borrow_unchecked(|wdt| {
        wdt.mr.modify(|_, w| w.wddis().set_bit())
    });

    // Clock initialization
    pac::PMC::borrow_unchecked(|pmc| {
        #[cfg(feature = "atsam4e")]
        pac::EFC::borrow_unchecked(|efc| {
            clock::init(pmc, efc);
        });
    
        #[cfg(feature = "atsam4s")]
        pac::EFC0::borrow_unchecked(|efc0| {
            pac::EFC1::borrow_unchecked(|efc1| {
                clock::init(pmc, efc0, efc1);
            });
        });
    });
}

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

#[cfg(feature = "atsam4e")]
borrow_unchecked!(WDT, PMC, EFC);

#[cfg(feature = "atsam4s")]
borrow_unchecked!(WDT, PMC, EFC0, EFC1);
