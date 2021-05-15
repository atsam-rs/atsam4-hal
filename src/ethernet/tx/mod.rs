mod descriptor;
use descriptor::TxDescriptor;

mod descriptor_block;
pub use descriptor_block::TxDescriptorBlock;

use super::{
    MTU,
    Transmitter,
    VolatileReadWrite,
};

pub enum TxError {
    WouldBlock,
}
