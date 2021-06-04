use super::{tx::Descriptor as TxDescriptor, DescriptorTableT, MTU};
use crate::pac::GMAC;

#[derive(Debug)]
pub enum Error {
    InvalidArgument,
}

pub struct Transmitter<'tx> {
    pub(super) descriptors: &'tx mut dyn DescriptorTableT<TxDescriptor>,
}

impl<'tx> Transmitter<'tx> {
    pub fn new(descriptors: &'tx mut dyn DescriptorTableT<TxDescriptor>) -> Self {
        Transmitter { descriptors }
    }

    pub fn send(&self, gmac: &GMAC, buffer: &[u8]) -> nb::Result<(), Error> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let buffer_length = buffer.len();
        if buffer_length > MTU {
            return Err(nb::Error::Other(Error::InvalidArgument));
        }

        let (next_descriptor, next_buffer) = self.descriptors.next_descriptor_pair();
        if !next_descriptor.read().used() {
            return Err(nb::Error::WouldBlock);
        }

        // Set the length on the buffer descriptor
        next_descriptor.modify(|w| w.set_buffer_size(buffer_length as u16));

        let mut descriptor_buffer = next_buffer.borrow_mut();
        descriptor_buffer[..buffer_length].clone_from_slice(&buffer);

        // Start the transmission
        Self::start_transmission(&gmac);

        Ok(())
    }

    fn start_transmission(gmac: &GMAC) {
        gmac.ncr.modify(|_, w| w.tstart().set_bit());
    }
}
