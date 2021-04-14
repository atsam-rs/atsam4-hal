use volatile::*;

#[repr(C, align(4))]
pub struct TxBufDescBlock<const TX_BUFFER_COUNT: usize> {
    descriptors: [TxBufDesc; TX_BUFFER_COUNT],
}

impl<const TX_BUFFER_COUNT: usize> TxBufDescBlock<TX_BUFFER_COUNT> {
    pub fn new() -> Self {
        let mut a = TxBufDescBlock {
            ..Default::default()
        };

        // Set the "last buffer" flag on the last descriptor in the block
        a.descriptors[a.descriptors.len() - 1].set_last_buffer();
        a
    }
}

impl<const TX_BUFFER_COUNT: usize> Default for TxBufDescBlock<TX_BUFFER_COUNT> {
    fn default() -> TxBufDescBlock<TX_BUFFER_COUNT> {
        TxBufDescBlock {
            descriptors: [TxBufDesc { word0: 0, word1: 0 }; TX_BUFFER_COUNT],
        }
    }
}

#[repr(C, align(8))]
#[derive(Clone, Copy, Default)]
pub struct TxBufDesc {
    pub word0: u32,
    pub word1: u32,
}

impl TxBufDesc {
    pub fn set_last_buffer(&mut self) {
        let mut word1 = Volatile::new(&mut self.word0);
        word1.update(|value| {
            *value |= 1 << 15;
        })
    }

    pub fn set_address(&mut self, address: *const u32) {
        let mut word0 = Volatile::new(&mut self.word0);
        word0.write(address as u32);
    }
}
