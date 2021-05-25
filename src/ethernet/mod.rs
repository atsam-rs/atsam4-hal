use crate::pac::GMAC;

mod controller;
pub use controller::*;

mod builder;
pub use builder::Builder;

mod eui48;
pub use eui48::Identifier as EthernetAddress;

mod phy;

#[cfg(feature = "smoltcp")]
mod smoltcp_support;

mod rx;
pub use rx::{Receiver, RxDescriptorTable};

mod tx;
pub use tx::{Transmitter, TxDescriptorTable};

mod volatile_read_write;
pub use volatile_read_write::VolatileReadWrite;

const MTU: usize = 1522;
