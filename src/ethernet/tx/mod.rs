use super::{MTU, VolatileReadWrite};

mod descriptor;
use descriptor::TxDescriptor;

mod descriptor_table;
pub use descriptor_table::TxDescriptorTable;

mod transmitter;
pub use transmitter::Transmitter;

pub enum TxError {
}

pub trait DescriptorTable {
    fn initialize(&mut self);
    fn base_address(&self) -> u32;
    fn next_descriptor(&self) -> &TxDescriptor;
    fn next_descriptor_pair(&mut self) -> (&mut TxDescriptor, &mut [u8]);
    fn consume_next_descriptor(&mut self);
}
