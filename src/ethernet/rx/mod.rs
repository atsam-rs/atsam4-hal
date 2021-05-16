mod descriptor;
use descriptor::RxDescriptor;

mod descriptor_block;
pub use descriptor_block::RxDescriptorBlock;

use super::{Receiver, VolatileReadWrite, MTU};

pub enum RxError {
    WouldBlock,
}
