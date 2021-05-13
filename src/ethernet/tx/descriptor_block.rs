use crate::{
    pac::GMAC,
};

use super::{
    DescriptorBlock,
    Transmitter,
    TxError,
    TxDescriptor,
    MTU,
};

pub struct TxDescriptorBlock<const COUNT: usize> {
    descriptors: DescriptorBlock<TxDescriptor, MTU, COUNT>
}

impl<const COUNT: usize> TxDescriptorBlock<COUNT> {
    pub fn new() -> Self {
        let tx = TxDescriptorBlock {
            descriptors: DescriptorBlock::new()
        };

        tx
    }

    pub fn setup_dma(&self, gmac: &GMAC) {
        gmac.tbqb.write(|w| unsafe { w.bits(self.descriptors.descriptor_table_address()) });
    }
}

impl<const COUNT: usize> Transmitter for TxDescriptorBlock<COUNT> {
    fn send<F: FnOnce(&mut [u8], usize)>(&mut self, length: usize, f: F) -> Result<(), TxError> {
        // Check if the next entry is still being used by the GMAC...if so, 
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_mut();
        if !next_descriptor.read().is_used() {
            return Err(TxError::WouldBlock);
        }

        // Set the length on the buffer descriptor
        next_descriptor.modify(|w| w
            .set_buffer_length(length)
        );

        // Call the closure to fill the buffer
        f(next_buffer, length);
        
        // Indicate to the GMAC that the entry is available for it to transmit
        next_descriptor.modify(|w| w
            .set_used() 
        );

        // This entry is now in use, indicate this.
        self.descriptors.increment_next_entry();

        Ok(())
    }
}
