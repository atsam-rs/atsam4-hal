pub trait DescriptorEntry {
    fn initialize(&mut self, address: *const u8);
    fn set_wrap(&mut self);
}

#[repr(C)]
pub struct DescriptorBlock<T:Copy + Default + DescriptorEntry, const MTU: usize, const COUNT: usize> {
    descriptors: [T; COUNT],
    buffers: [[u8; MTU]; COUNT],

    next_entry: usize,  // Index of next entry to read/write
}

impl<T:Copy + Default + DescriptorEntry, const MTU: usize, const COUNT: usize> DescriptorBlock<T, MTU, COUNT> {
    pub fn new() -> Self {
        let mut block = DescriptorBlock::<T, MTU, COUNT> {
            descriptors: [Default::default(); COUNT],
            buffers: [[0; MTU]; COUNT],
            next_entry: 0,
        };

        // Set the address inside each descriptor to point to the associated buffer
        for i in 0..COUNT {
            block.descriptors[i].initialize(&block.buffers[i][0])
        }

        block.descriptors[COUNT - 1].set_wrap();
        block
    }

    pub fn next_mut(&mut self) -> (&mut T, &mut [u8]) {
        (&mut self.descriptors[self.next_entry], &mut self.buffers[self.next_entry])
    }

    pub fn increment_next_entry(&mut self) {
        if self.next_entry == COUNT - 1 {
            self.next_entry = 0;
        } else {
            self.next_entry += 1;
        }
    }

    pub fn descriptor_table_address(&self) -> u32 {
        let address:*const T = &self.descriptors[0];
        let a = address as u32;
        if a & 0x0000_0003 != 0 {
            panic!("Unaligned buffer address in descriptor table")
        }
        a
    }
}
