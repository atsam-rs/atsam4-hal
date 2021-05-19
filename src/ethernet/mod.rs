mod controller;
pub use controller::*;

mod builder;
pub use builder::Builder;

mod eui48;
pub use eui48::Identifier as EthernetAddress;

mod phy;

#[cfg(feature = "smoltcp")]
mod smoltcp_support;

mod tx;
pub use tx::{TxDescriptorBlock, TxError};

mod rx;
pub use rx::{RxDescriptorBlock, RxError};

mod volatile_read_write;
pub use volatile_read_write::VolatileReadWrite;

const MTU: usize = 1522;

pub trait Receiver {
    #[cfg(not(feature = "smoltcp"))]
    fn receive<R, F: FnOnce(&mut [u8]) -> Result<R, RxError>>(
        &mut self,
        f: F,
    ) -> Result<R, RxError>
    where
        Self: Sized;

    #[cfg(feature = "smoltcp")]
    fn receive<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &mut self,
        f: F,
    ) -> Result<R, smoltcp::Error>
    where
        Self: Sized;
}

pub trait Transmitter {
    #[cfg(not(feature = "smoltcp"))]
    fn send<R, F: FnOnce(&mut [u8], u16) -> Result<R, TxError>>(
        &mut self,
        size: u16,
        f: F,
    ) -> Result<R, TxError>
    where
        Self: Sized;

    #[cfg(feature = "smoltcp")]
    fn send<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &mut self,
        size: usize,
        f: F,
    ) -> Result<R, smoltcp::Error>
    where
        Self: Sized;
}
