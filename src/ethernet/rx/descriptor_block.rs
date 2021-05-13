use crate::pac::GMAC;
use super::{
    DescriptorBlock,
    Receiver,
    RxError,
    RxDescriptor,
    MTU,
};

pub struct RxDescriptorBlock<const COUNT: usize> {
    descriptors: DescriptorBlock<RxDescriptor, MTU, COUNT>
}

impl<const COUNT: usize> RxDescriptorBlock<COUNT> {
    pub fn new() -> Self {
        let rx = RxDescriptorBlock {
            descriptors: DescriptorBlock::new()
        };

        rx
    }

    pub fn setup_dma(&self, gmac: &GMAC) {
        gmac.rbqb.write(|w| unsafe { w.bits(self.descriptors.descriptor_table_address()) });
    }
}

impl<const COUNT: usize> Receiver for RxDescriptorBlock<COUNT> {
    fn receive<F: FnOnce(&mut [u8], usize)>(&mut self, f: F) -> Result<(), RxError> {
        // Check if the next entry is still being used by the GMAC...if so, 
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_mut();
        let descriptor_properties = next_descriptor.read();
        if !descriptor_properties.is_owned() {
            return Err(RxError::WouldBlock);
        }

        let length = descriptor_properties.buffer_length();

        // Call the closure to copy data out of the buffer
        f(next_buffer, length as usize);

        // Indicate that the descriptor is no longer owned by software and is available
        // for the GMAC to write into.
        next_descriptor.modify(|w| w.clear_owned());

        // This entry has been consumed, indicate this.
        self.descriptors.increment_next_entry();

        Ok(())
    }
}
