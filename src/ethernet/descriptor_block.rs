pub trait DescriptorEntry {
    fn initialize(&mut self, address: *const u8);
    fn set_wrap(&mut self);
}

#[repr(C)]
pub struct DescriptorBlock<T:Copy + Default + DescriptorEntry, const COUNT: usize, const MTU: usize> {
    descriptors: [T; COUNT],
    buffers: [[u8; MTU]; COUNT],

    next_entry: usize,  // Index of next entry to read/write
}

impl<T:Copy + Default + DescriptorEntry, const COUNT: usize, const MTU: usize> DescriptorBlock<T, COUNT, MTU> {
    pub fn new() -> Self {
        let mut block = DescriptorBlock::<T, COUNT, MTU> {
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
}
