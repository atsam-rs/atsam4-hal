use crate::pac::GMAC;
use super::{DescriptorTable, MTU, TxError};

trait BlockingError {

}

pub struct Transmitter<'tx> {
    descriptors: &'tx mut dyn DescriptorTable,
}

impl<'tx> Transmitter<'tx> {
    pub fn new(descriptors: &'tx mut dyn DescriptorTable) -> Self {
        descriptors.initialize();
        Transmitter {
            descriptors,
        }
    }

    pub fn send<R, F: FnOnce(&mut [u8]) -> nb::Result<R, TxError>>(
        &mut self,
        gmac: &GMAC, 
        size: usize,
        f: F,
    ) -> nb::Result<R, TxError> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        debug_assert!(size <= MTU);

        let (next_descriptor, next_buffer) = self.descriptors.next_descriptor_pair();
        if !next_descriptor.read().is_used() {
            return Err(nb::Error::WouldBlock);
        }

        // Set the length on the buffer descriptor
        next_descriptor.modify(|w| w.set_buffer_size(size as u16));

        // Call the closure to fill the buffer
        let r = f(&mut next_buffer[0..size]);

        // Indicate to the GMAC that the entry is available for it to transmit
        next_descriptor.modify(|w| w.clear_used());

        // This entry is now in use, indicate this.
        self.descriptors.consume_next_descriptor();

        // Start the transmission
        gmac.ncr.modify(|_, w| w.tstart().set_bit());

        r
    }

    pub fn send_smoltcp<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &mut self,
        gmac: &GMAC,
        size: usize,
        f: F,
    ) -> Result<R, smoltcp::Error> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        debug_assert!(size <= MTU);

        let (next_descriptor, next_buffer) = self.descriptors.next_descriptor_pair();
        if !next_descriptor.read().is_used() {
            return Err(smoltcp::Error::Exhausted);
        }

        // Set the length on the buffer descriptor
        next_descriptor.modify(|w| w.set_buffer_size(size as u16));

        // Call the closure to fill the buffer
        let r = f(&mut next_buffer[0..size]);

        // Indicate to the GMAC that the entry is available for it to transmit
        next_descriptor.modify(|w| w.clear_used());

        // This entry is now in use, indicate this.
        self.descriptors.consume_next_descriptor();

        // Start the transmission
        gmac.ncr.modify(|_, w| w.tstart().set_bit());

        r
    }
}
