use vcell::VolatileCell;
use super::descriptor_block::DescriptorEntry;

pub struct RxDescriptorReader(u32, u32);
impl RxDescriptorReader {
    // Word 0 bits
    pub fn is_wrap(&self) -> bool {
        self.0 & (1 << 0) != 0x0
    }

    pub fn is_owned(&self) -> bool {
        self.0 & (1 << 1) != 0x0
    }

    // Word 1 bits
    pub fn frame_length_in_bytes(&self) -> u16 {
        //!todo - If jumbo frames are enabled, this needs to take into account the 13th bit as well.
        (self.1 & 0x0000_0FFF) as u16
    }

    pub fn is_start_of_frame(&self) -> bool {
        self.1 & (1 << 14) != 0
    }

    pub fn is_end_of_frame(&self) -> bool {
        self.1 & (1 << 15) != 0
    }
}

pub struct RxDescriptorWriter(u32, u32);
impl RxDescriptorWriter {
    pub fn set_address(self, address: *const u8) -> Self {
        if (address as u32) & 0x0000_0003 != 0 {
            panic!("Specified address is not 32 bit aligned");
        }
        RxDescriptorWriter(self.0 | ((address as u32) & !0x03), self.1)
    }

    pub fn set_owned(self) -> Self {
        RxDescriptorWriter(self.0 | 0x0000_0001, self.1)
    }

    pub fn clear_owned(self) -> Self {
        RxDescriptorWriter(self.0 & !0x0000_0001, self.1)
    }

    pub fn set_wrap(self) -> Self {
        RxDescriptorWriter(self.0 | 0x0000_0002, self.1)
    }

    pub fn clear_wrap(self) -> Self {
        RxDescriptorWriter(self.0 & !0x0000_0002, self.1)
    }
}

#[repr(C)]
pub struct RxDescriptor {
    word0: VolatileCell<u32>,
    word1: VolatileCell<u32>,
}

impl RxDescriptor {
    pub fn read(&self) -> RxDescriptorReader {
        RxDescriptorReader(self.word0.get(), self.word1.get())
    }

    pub fn modify<F: FnOnce(RxDescriptorWriter) -> RxDescriptorWriter>(&mut self, f: F) {
        let w = RxDescriptorWriter(self.word0.get(), self.word1.get());
        let result = f(w);
        self.word0.set(result.0);
        self.word1.set(result.1);
    }
}

impl Default for RxDescriptor {
    fn default() -> Self {
        RxDescriptor {
            word0: VolatileCell::new(0),
            word1: VolatileCell::new(0),
        }
    }
}

impl DescriptorEntry for RxDescriptor {
    fn set_wrap(&mut self) {
        self.modify(|w| w.set_wrap());
    }

    fn set_address(&mut self, address: *const u8) {
        self.modify(|w| w.set_address(address));
    }
}
