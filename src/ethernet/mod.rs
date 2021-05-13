mod controller;
pub use controller::*;

mod descriptor_block;
pub use descriptor_block::{DescriptorBlock, DescriptorEntry};

mod builder;
pub use builder::Builder;

mod eui48;
pub use eui48::Identifier as EthernetAddress;

mod phy;

#[cfg(feature = "smoltcp")]
mod smoltcp;

mod tx;
pub use tx::{TxDescriptorBlock, TxError};

mod rx;
pub use rx::{RxDescriptorBlock, RxError};

mod volatile_read_write;
pub use volatile_read_write::VolatileReadWrite;

const MTU: usize = 1500;

pub trait Receiver {
    fn receive<F: FnOnce(&mut [u8], u16)>(&mut self, f: F) -> Result<(), RxError> where Self: Sized;
}

pub trait Transmitter {
    fn send<F: FnOnce(&mut [u8], u16)>(&mut self, size: u16, f: F) -> Result<(), TxError> where Self: Sized;
}
