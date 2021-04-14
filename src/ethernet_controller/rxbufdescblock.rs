use volatile::*;

#[repr(C, align(4))]
pub struct RxBufDescBlock<const RX_BUFFER_COUNT: usize> {
    descriptors: [RxBufDesc; RX_BUFFER_COUNT],
}

impl<const RX_BUFFER_COUNT: usize> RxBufDescBlock<RX_BUFFER_COUNT> {
    pub fn new() -> Self {
        let mut a = RxBufDescBlock {
            ..Default::default()
        };

        // Set the wrap flag on the last descriptor in the block
        a.descriptors[a.descriptors.len() - 1].set_wrap();
        a
    }
}

impl<const RX_BUFFER_COUNT: usize> Default for RxBufDescBlock<RX_BUFFER_COUNT> {
    fn default() -> RxBufDescBlock<RX_BUFFER_COUNT> {
        RxBufDescBlock {
            descriptors: [RxBufDesc { word0: 0, word1: 0 }; RX_BUFFER_COUNT],
        }
    }
}

#[repr(C, align(8))]
#[derive(Clone, Copy, Default)]
pub struct RxBufDesc {
    pub word0: u32,
    pub word1: u32,
}

impl RxBufDesc {
    pub fn set_owned(&mut self) {
        let mut word0 = Volatile::new(&mut self.word0);
        word0.update(|value| {
            *value |= 0x01;
        })
    }

    pub fn is_owned(&self) -> bool {
        let word0 = Volatile::new(&self.word0);
        word0.read() & 0x01 != 0
    }

    pub fn set_wrap(&mut self) {
        let mut word0 = Volatile::new(&mut self.word0);
        word0.update(|value| {
            *value |= 0x02;
        })
    }

    pub fn set_address(&mut self, address: *const u32) {
        if (address as u32) & 0x03 != 0 {
            panic!("Specified address is not 32 bit aligned");
        }

        let mut word0 = Volatile::new(&mut self.word0);
        word0.update(|value| {
            *value |= (address as u32) & !0x03;
        })
    }
}
