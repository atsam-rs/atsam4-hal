use super::{DescriptorTable, RxDescriptor, MTU};

// In order to keep the buffers 32 bit aligned (required by the hardware), we adjust
// the size here to be the next 4 byte multiple greater than the requested MTU.
const BUFFERSIZE: usize = (MTU & !3) + 4;

#[repr(C)]
pub struct RxDescriptorTable<const COUNT: usize> {
    descriptors: [RxDescriptor; COUNT],
    buffers: [[u8; BUFFERSIZE]; COUNT],

    next_entry: usize, // Index of next entry to read/write
}

impl<const COUNT: usize> RxDescriptorTable<COUNT> {
    pub const fn const_default() -> Self {
        let rx = RxDescriptorTable {
            descriptors: [RxDescriptor::const_default(); COUNT],
            buffers: [[0; BUFFERSIZE]; COUNT],
            next_entry: 0,
        };

        rx
    }

    fn increment_next_entry(&mut self) {
        if self.next_entry == COUNT - 1 {
            self.next_entry = 0;
        } else {
            self.next_entry += 1;
        }
    }

    fn next(&self) -> (&RxDescriptor, &[u8]) {
        (
            &self.descriptors[self.next_entry],
            &self.buffers[self.next_entry],
        )
    }

    fn next_mut(&mut self) -> (&mut RxDescriptor, &mut [u8]) {
        (
            &mut self.descriptors[self.next_entry],
            &mut self.buffers[self.next_entry],
        )
    }
}
/*
impl<const COUNT: usize> Receiver for RxDescriptorTable<COUNT> {
    fn initialize(&mut self, gmac: &GMAC) {
        let mut i = 0;
        for descriptor in self.descriptors.iter_mut() {
            let buffer_address = &self.buffers[i][0];
            descriptor.initialize(buffer_address);
            i += 1;
        }

        self.descriptors[COUNT - 1].modify(|w| w.set_wrap());

        gmac.rbqb
            .write(|w| unsafe { w.bits(self.descriptor_table_address()) });
    }

    fn can_receive(&self) -> bool {
        let (next_descriptor, _) = self.next();
        let descriptor_properties = next_descriptor.read();
        descriptor_properties.is_owned()
    }

    #[cfg(not(feature = "smoltcp"))]
    fn receive<R, F: FnOnce(&mut [u8]) -> Result<R, RxError>>(
        &mut self,
        f: F,
    ) -> Result<R, RxError> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.next_mut();
        let descriptor_properties = next_descriptor.read();
        if !descriptor_properties.is_owned() {
            return Err(RxError::WouldBlock);
        }

        let buffer_size = descriptor_properties.buffer_size() as usize;

        // Call the closure to copy data out of the buffer
        let r = f(&mut next_buffer[0..buffer_size]);

        // Indicate that the descriptor is no longer owned by software and is available
        // for the GMAC to write into.
        next_descriptor.modify(|w| w.clear_owned());

        // This entry has been consumed, indicate this.
        self.descriptors.increment_next_entry();

        r
    }

    #[cfg(feature = "smoltcp")]
    fn receive<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &mut self,
        f: F,
    ) -> Result<R, smoltcp::Error> {
        // Check if the next entry is still being used by the GMAC...if so,
        // indicate there's no more entries and the client has to wait for one to
        // become available.
        let (next_descriptor, next_buffer) = self.next_mut();
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
        self.increment_next_entry();

        r
    }
}
*/
impl<const COUNT: usize> DescriptorTable for RxDescriptorTable<COUNT> {
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
        let address: *const RxDescriptor = &self.descriptors[0];
        let a = address as u32;
        if a & 0x0000_0003 != 0 {
            panic!("Unaligned buffer address in descriptor table")
        }
        a
    }

    fn next_descriptor(&self) -> &RxDescriptor {
        &self.descriptors[self.next_entry]
    }

    fn next_descriptor_pair(&mut self) -> (&mut RxDescriptor, &mut [u8]) {
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
