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

pub extern crate embedded_hal;
pub use embedded_hal::digital::v2::*;

#[cfg(feature = "atsam4e16e")]
pub use atsam4e16e_pac as pac;

#[cfg(feature = "atsam4sd32c")]
pub use atsam4sd32c_pac as pac;

use cortex_m_rt::pre_init;
use core::mem;

pub mod clock;
pub mod delay;
pub mod gpio;
pub mod static_memory_controller;
pub mod serial;
pub mod time;

// peripheral initialization
#[pre_init]
unsafe fn pre_init() {
    // Clock initialization
    pac::PMC::borrow_unchecked(|pmc| {
        initialize_clock(pmc);
    });
}

unsafe fn initialize_clock(pmc: &mut pac::PMC) {
    // Disable the watchdog timer (it starts running at reset)
    pac::WDT::borrow_unchecked(|wdt| {
        wdt.mr.modify(|_, w| w.wddis().set_bit())
    });

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
}

/// Borrows a peripheral without checking if it has already been taken
unsafe trait BorrowUnchecked {
    fn borrow_unchecked<T>(f: impl FnOnce(&mut Self) -> T) -> T;
}

unsafe impl BorrowUnchecked for pac::WDT {
    fn borrow_unchecked<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        let mut p = unsafe { mem::transmute(()) };
        f(&mut p)
    }
}

unsafe impl BorrowUnchecked for pac::PMC {
    fn borrow_unchecked<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        let mut p = unsafe { mem::transmute(()) };
        f(&mut p)
    }
}

#[cfg(feature = "atsam4e")]
unsafe impl BorrowUnchecked for pac::EFC {
    fn borrow_unchecked<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        let mut p = unsafe { mem::transmute(()) };
        f(&mut p)
    }
}

#[cfg(feature = "atsam4s")]
unsafe impl BorrowUnchecked for pac::EFC0 {
    fn borrow_unchecked<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        let mut p = unsafe { mem::transmute(()) };
        f(&mut p)
    }
}

#[cfg(feature = "atsam4s")]
unsafe impl BorrowUnchecked for pac::EFC1 {
    fn borrow_unchecked<T>(f: impl FnOnce(&mut Self) -> T) -> T {
        let mut p = unsafe { mem::transmute(()) };
        f(&mut p)
    }
}
