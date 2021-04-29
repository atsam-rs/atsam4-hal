mod controller;
pub use controller::*;
mod descriptor_block;
mod builder;
pub use builder::Builder;
mod eui48;
pub use eui48::Identifier as EthernetAddress;
mod phy;

#[cfg(feature = "smoltcp")]
mod smoltcp;

const MTU: usize = 1522;

mod tx;
pub use tx::TxError;

mod rx;
pub use rx::RxError;

mod volatile_read_write;
pub use volatile_read_write::VolatileReadWrite;
