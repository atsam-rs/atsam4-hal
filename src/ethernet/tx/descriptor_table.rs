use super::{DescriptorTable, TxDescriptor, MTU};

// In order to keep the buffers 32 bit aligned (required by the hardware), we adjust
// the size here to be the next 4 byte multiple greater than the requested MTU.
const BUFFERSIZE: usize = (MTU & !3) + 4;

#[repr(C)]
pub struct TxDescriptorTable<const COUNT: usize> {
    descriptors: [TxDescriptor; COUNT],
    buffers: [[u8; BUFFERSIZE]; COUNT],

    next_entry: usize, // Index of next entry to read/write
}

impl<'a, const COUNT: usize> TxDescriptorTable<COUNT> {
    pub const fn const_default() -> Self {
        let tx = TxDescriptorTable {
            descriptors: [TxDescriptor::const_default(); COUNT],
            buffers: [[0; BUFFERSIZE]; COUNT],
            next_entry: 0,
        };

        tx
    }

    fn increment_next_entry(&mut self) {
        if self.next_entry == COUNT - 1 {
            self.next_entry = 0;
        } else {
            self.next_entry += 1;
        }
    }

    fn next(&self) -> (&TxDescriptor, &[u8]) {
        (
            &self.descriptors[self.next_entry],
            &self.buffers[self.next_entry],
        )
    }

    fn next_mut(&mut self) -> (&mut TxDescriptor, &mut [u8]) {
        (
            &mut self.descriptors[self.next_entry],
            &mut self.buffers[self.next_entry],
        )
    }
}
/*
impl<const COUNT: usize> Transmitter for TxDescriptorBlock<COUNT> {
    fn initialize(&mut self, gmac: &GMAC) {
        let mut i = 0;
        for descriptor in self.descriptors.iter_mut() {
            let buffer_address = &self.buffers[i][0];
            descriptor.initialize(buffer_address);
            i += 1;
        }

        self.descriptors[COUNT - 1].modify(|w| w.set_wrap());

        gmac.tbqb
            .write(|w| unsafe { w.bits(self.descriptor_table_address()) });
    }
    
    #[cfg(not(feature = "smoltcp"))]
    fn send<R, F: FnOnce(&mut [u8], u16) -> Result<R, TxError>>(
        &mut self,
        size: u16,
        f: F,
    ) -> Result<R, TxError> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.descriptors.next_mut();
        if !next_descriptor.read().is_used() {
            return Err(TxError::WouldBlock);
        }

        // Set the length on the buffer descriptor
        next_descriptor.modify(|w| w.set_buffer_size(size));

        // Call the closure to fill the buffer
        let r = f(next_buffer, size);

        // Indicate to the GMAC that the entry is available for it to transmit
        next_descriptor.modify(|w| w.set_used());

        // This entry is now in use, indicate this.
        self.descriptors.increment_next_entry();

        r
    }

    #[cfg(feature = "smoltcp")]
    fn send<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &mut self,
        size: usize,
        f: F,
    ) -> Result<R, smoltcp::Error> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        debug_assert!(size <= MTU);

        let (next_descriptor, next_buffer) = self.next_mut();
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
        self.increment_next_entry();

        r
    }
}
*/
impl<const COUNT: usize> DescriptorTable for TxDescriptorTable<COUNT> {
    fn initialize(&mut self) {
        let mut i = 0;
        for descriptor in self.descriptors.iter_mut() {
            let buffer_address = &self.buffers[i][0];
            descriptor.initialize(buffer_address);
            i += 1;
        }

        self.descriptors[COUNT - 1].modify(|w| w.set_wrap());        
    }
    
    fn base_address(&self) -> u32 {
        let address: *const TxDescriptor = &self.descriptors[0];
        let a = address as u32;
        if a & 0x0000_0003 != 0 {
            panic!("Unaligned buffer address in descriptor table")
        }
        a
    }

    fn next_descriptor(&self) -> &TxDescriptor {
        &mut self.descriptors[self.next_entry]
    }

    fn next_descriptor_pair(&mut self) -> (&mut TxDescriptor, &mut [u8]) {
        (&mut self.descriptors[self.next_entry], &mut self.buffers[self.next_entry])
    }

    fn consume_next_descriptor(&mut self) {
        if self.next_entry == COUNT - 1 {
            self.next_entry = 0;
        } else {
            self.next_entry += 1;
        }
    }
}
