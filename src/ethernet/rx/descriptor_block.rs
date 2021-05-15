use crate::pac::GMAC;
use super::{
    Receiver,
    RxDescriptor,
    MTU,
};

#[cfg(not(feature = "smoltcp"))]
use super::RxError;

pub struct RxDescriptorBlock<const COUNT: usize> {
    descriptors: [RxDescriptor; COUNT],
    buffers: [[u8; MTU]; COUNT],

    next_entry: usize,  // Index of next entry to read/write
}

impl<const COUNT: usize> RxDescriptorBlock<COUNT> {
    pub const fn const_default() -> Self {
        let rx = RxDescriptorBlock {
            descriptors: [RxDescriptor::const_default(); COUNT],
            buffers: [[0; MTU]; COUNT],
            next_entry: 0,
        };

        rx
    }

    pub fn initialize(&mut self, gmac: &GMAC) {
        let mut i = 0;
        for descriptor in self.descriptors.iter_mut() {   
            let buffer_address = &self.buffers[i][0];         
            descriptor.modify(|w| {
                w.set_address(buffer_address)
            });
            i += 1;
        }

        self.descriptors[COUNT - 1].modify(|w| w.set_wrap());

        gmac.rbqb.write(|w| unsafe { w.bits(self.descriptor_table_address()) });
    }

    fn descriptor_table_address(&self) -> u32 {
        let address:*const RxDescriptor = &self.descriptors[0];
        let a = address as u32;
        if a & 0x0000_0003 != 0 {
            panic!("Unaligned buffer address in descriptor table")
        }
        a
    }

    fn increment_next_entry(&mut self) {
        if self.next_entry == COUNT - 1 {
            self.next_entry = 0;
        } else {
            self.next_entry += 1;
        }
    }

    fn next_mut(&mut self) -> (&mut RxDescriptor, &mut [u8]) {
        (&mut self.descriptors[self.next_entry], &mut self.buffers[self.next_entry])
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
        let (next_descriptor, next_buffer) = self.next_mut();
        let descriptor_properties = next_descriptor.read();
        if !descriptor_properties.is_owned() {
            return Err(smoltcp::Error::Exhausted);
        }

//        let size = descriptor_properties.buffer_size();

        // Call the closure to copy data out of the buffer
        let r = f(next_buffer);

        // Indicate that the descriptor is no longer owned by software and is available
        // for the GMAC to write into.
        next_descriptor.modify(|w| w.clear_owned());

        // This entry has been consumed, indicate this.
        self.increment_next_entry();

        r
    }}
