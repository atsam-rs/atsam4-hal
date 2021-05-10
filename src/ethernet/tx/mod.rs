mod descriptor;
use descriptor::TxDescriptor;

mod descriptor_block;
pub use descriptor_block::TxDescriptorBlock;

use super::{
    DescriptorBlock,
    DescriptorEntry,
    MTU,
    Transmitter,
    VolatileReadWrite,
};

pub enum TxError {
    WouldBlock,
}
