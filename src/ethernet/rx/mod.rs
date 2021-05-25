use super::{VolatileReadWrite, MTU};

mod descriptor;
use descriptor::RxDescriptor;

mod descriptor_table;
pub use descriptor_table::RxDescriptorTable;

mod receiver;
pub use receiver::Receiver;

pub trait DescriptorTable {
    fn initialize(&mut self);
    fn base_address(&self) -> u32;
    fn next_descriptor(&self) -> &RxDescriptor;
    fn next_descriptor_pair(&mut self) -> (&mut RxDescriptor, &mut [u8]);
    fn consume_next_descriptor(&mut self);
}

pub enum RxError {
}
