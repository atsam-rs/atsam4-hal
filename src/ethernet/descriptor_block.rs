pub trait DescriptorEntry {
    fn set_address(&mut self, address: *const u8);
    fn set_wrap(&mut self);
}

#[repr(C)]
struct DescriptorBlock<T:Copy + Default + DescriptorEntry, const COUNT: usize, const MTU: usize> {
    descriptors: [T; COUNT],
    buffers: [[u8; MTU]; COUNT],
}

impl<T:Copy + Default + DescriptorEntry, const COUNT: usize, const MTU: usize> DescriptorBlock<T, COUNT, MTU> {
    pub fn new() -> Self {
        let mut block = DescriptorBlock::<T, COUNT, MTU> {
            descriptors: [T::default(); COUNT],
            buffers: [[0; MTU]; COUNT],
        };

        // Set the address inside each descriptor to point to the associated buffer
        for i in 0..COUNT {
            block.descriptors[i].set_address(&block.buffers[i][0])
        }

        block.descriptors[COUNT - 1].set_wrap();
        block
    }
}
