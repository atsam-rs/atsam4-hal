mod descriptor;
use descriptor::TxDescriptor;

mod descriptor_block;
pub use descriptor_block::TxDescriptorBlock;

use super::{Transmitter, VolatileReadWrite, MTU};

pub enum TxError {
    WouldBlock,
}
