use crate::pac::GMAC;
use super::{DescriptorTable, RxError};

pub struct Receiver<'rx> {
    descriptors: &'rx mut dyn DescriptorTable,
}

impl<'rx> Receiver<'rx> {
    pub fn new(descriptors: &'rx mut dyn DescriptorTable) -> Self {
        descriptors.initialize();
        Receiver {
            descriptors,
        }
    }

    pub fn can_receive(&self) -> bool {
        self.descriptors.next_descriptor().read().is_owned()
    }

    pub fn receive<R, F: FnOnce(&mut [u8]) -> nb::Result<R, RxError>>(
        &mut self,
        f: F,
    ) -> nb::Result<R, RxError> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_descriptor_pair();
        let descriptor_properties = next_descriptor.read();
        if !descriptor_properties.is_owned() {
            return Err(nb::Error::WouldBlock);
        }

        let buffer_size = descriptor_properties.buffer_size() as usize;

        // Call the closure to copy data out of the buffer
        let r = f(&mut next_buffer[0..buffer_size]);

        // Indicate that the descriptor is no longer owned by software and is available
        // for the GMAC to write into.
        next_descriptor.modify(|w| w.clear_owned());

        // This entry has been consumed, indicate this.
        self.descriptors.consume_next_descriptor();

        r
    }

    pub fn receive_smoltcp<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &mut self,
        f: F,
    ) -> Result<R, smoltcp::Error> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_descriptor_pair();
        let descriptor_properties = next_descriptor.read();
        if !descriptor_properties.is_owned() {
            return Err(smoltcp::Error::Exhausted);
        }

        let buffer_size = descriptor_properties.buffer_size() as usize;

        // Call the closure to copy data out of the buffer
        let r = f(&mut next_buffer[0..buffer_size]);

        // Indicate that the descriptor is no longer owned by software and is available
        // for the GMAC to write into.
        next_descriptor.modify(|w| w.clear_owned());

        // This entry has been consumed, indicate this.
        self.descriptors.consume_next_descriptor();

        r
    }
}
