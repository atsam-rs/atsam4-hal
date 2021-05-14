use crate::pac::GMAC;
use super::{
    DescriptorBlock,
    Receiver,
    RxDescriptor,
    MTU,
};

#[cfg(not(feature = "smoltcp"))]
use super::RxError;

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
    #[cfg(not(feature = "smoltcp"))]
    fn receive<R, F: FnOnce(&mut [u8]) -> Result<R, RxError>>(&mut self, f: F) -> Result<R, RxError> {
        // Check if the next entry is still being used by the GMAC...if so, 
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_mut();
        let descriptor_properties = next_descriptor.read();
        if !descriptor_properties.is_owned() {
            return Err(RxError::WouldBlock);
        }

        let size = descriptor_properties.buffer_size();

        // Call the closure to copy data out of the buffer
        let r = f(next_buffer);

        // Indicate that the descriptor is no longer owned by software and is available
        // for the GMAC to write into.
        next_descriptor.modify(|w| w.clear_owned());

        // This entry has been consumed, indicate this.
        self.descriptors.increment_next_entry();

        r
    }

    #[cfg(feature = "smoltcp")]
    fn receive<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(&mut self, f: F) -> Result<R, smoltcp::Error> {
        // Check if the next entry is still being used by the GMAC...if so, 
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_mut();
        let descriptor_properties = next_descriptor.read();
        if !descriptor_properties.is_owned() {
            return Err(smoltcp::Error::Exhausted);
        }

        let size = descriptor_properties.buffer_size();

        // Call the closure to copy data out of the buffer
        let r = f(next_buffer);

        // Indicate that the descriptor is no longer owned by software and is available
        // for the GMAC to write into.
        next_descriptor.modify(|w| w.clear_owned());

        // This entry has been consumed, indicate this.
        self.descriptors.increment_next_entry();

        r
    }}
