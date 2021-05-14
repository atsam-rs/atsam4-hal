use crate::{
    pac::GMAC,
};

use super::{
    DescriptorBlock,
    Transmitter,
    TxDescriptor,
    MTU,
};

#[cfg(not(feature = "smoltcp"))]
use super::TxError;

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
    #[cfg(not(feature = "smoltcp"))]
    fn send<R, F: FnOnce(&mut [u8], u16) -> Result<R, TxError>>(&mut self, size: u16, f: F) -> Result<R, TxError> {
        // Check if the next entry is still being used by the GMAC...if so, 
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_mut();
        if !next_descriptor.read().is_used() {
            return Err(TxError::WouldBlock);
        }

        // Set the length on the buffer descriptor
        next_descriptor.modify(|w| w
            .set_buffer_size(size)
        );

        // Call the closure to fill the buffer
        let r = f(next_buffer, size);
        
        // Indicate to the GMAC that the entry is available for it to transmit
        next_descriptor.modify(|w| w
            .set_used() 
        );

        // This entry is now in use, indicate this.
        self.descriptors.increment_next_entry();

        r
    }

    #[cfg(feature = "smoltcp")]
    fn send<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(&mut self, size: usize, f: F) -> Result<R, smoltcp::Error> {
        // Check if the next entry is still being used by the GMAC...if so, 
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_mut();
        if !next_descriptor.read().is_used() {
            return Err(smoltcp::Error::Exhausted);
        }

        // Set the length on the buffer descriptor
        next_descriptor.modify(|w| w
            .set_buffer_size(size as u16)
        );

        // Call the closure to fill the buffer
        let r = f(next_buffer);
        
        // Indicate to the GMAC that the entry is available for it to transmit
        next_descriptor.modify(|w| w
            .set_used() 
        );

        // This entry is now in use, indicate this.
        self.descriptors.increment_next_entry();

        r
    }
}
