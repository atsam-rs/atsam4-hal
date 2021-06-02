use heapless::Vec;
use super::MTU;

pub trait DescriptorT {
    fn new(buffer_address: *const u8, last_entry: bool) -> Self;
}

pub trait DescriptorTableT<DESCRIPTOR> {
    fn initialize(&mut self);
    fn base_address(&self) -> u32;
    fn next_descriptor(&self) -> &DESCRIPTOR;
    fn next_descriptor_pair(&mut self) -> (&mut DESCRIPTOR, &mut [u8]);
    fn consume_next_descriptor(&mut self);
}

// In order to keep the buffers 32 bit aligned (required by the hardware), we adjust
// the size here to be the next 4 byte multiple greater than the requested MTU.
const BUFFERSIZE: usize = (MTU & !3) + 4;

#[repr(C)]
pub struct DescriptorTable<DESCRIPTOR, const COUNT:usize> {
    descriptors: Vec<DESCRIPTOR, COUNT>,
    buffers: [[u8; BUFFERSIZE]; COUNT],

    next_entry: usize, // Index of next entry to read/write
}

impl<DESCRIPTOR, const COUNT:usize> DescriptorTable<DESCRIPTOR, COUNT> {
    pub const fn new() -> Self {
        DescriptorTable {
            descriptors: Vec::new(),
            buffers: [[0; BUFFERSIZE]; COUNT],
            next_entry: 0,
        }
    }
}

impl<DESCRIPTOR: DescriptorT, const COUNT:usize> DescriptorTableT<DESCRIPTOR> for DescriptorTable<DESCRIPTOR, COUNT> {
    fn initialize(&mut self) {
        self.descriptors.truncate(0);
        for i in 0..COUNT {
            let buffer_address = &self.buffers[i][0];
            let descriptor = DESCRIPTOR::new(buffer_address, i == COUNT - 1);
            self.descriptors.push(descriptor).ok();
        }
    }

    fn base_address(&self) -> u32 {
        let address: *const DESCRIPTOR = &self.descriptors[0];
        let a = address as u32;
        if a & 0x0000_0003 != 0 {
            panic!("Unaligned buffer address in descriptor table")
        }
        a
    }

    fn next_descriptor(&self) -> &DESCRIPTOR {
        &self.descriptors[self.next_entry]
    }

    fn next_descriptor_pair(&mut self) -> (&mut DESCRIPTOR, &mut [u8]) {
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
