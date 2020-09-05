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

#![deny(missing_docs)]
#![deny(warnings)]
#![no_std]

pub mod common;

#[cfg(feature = "atsam4e16e")]
pub use atsam4e16e_pac as pac;

#[cfg(feature = "atsam4e")]
pub mod sam4e;
#[cfg(feature = "atsam4e")]
pub use self::sam4e::*;

#[cfg(feature = "atsam4e16e")]
pub mod sam4e16e;
#[cfg(feature = "atsam4e16e")]
pub use self::sam4e16e::*;
