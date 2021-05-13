use super::{
    DescriptorEntry,
    VolatileReadWrite,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RxDescriptor {
    // NOTE: Only read or write these fields using volatile operations
    word0: u32,
    word1: u32,
}

impl RxDescriptor {
    pub fn read(&self) -> RxDescriptorReader {
        RxDescriptorReader(self.word0.read_volatile(), self.word1.read_volatile())
    }

    pub fn modify<F: FnOnce(RxDescriptorWriter) -> RxDescriptorWriter>(&mut self, f: F) {
        let w = RxDescriptorWriter(self.word0.read_volatile(), self.word1.read_volatile());
        let result = f(w);
        self.word0.write_volatile(result.0);
        self.word1.write_volatile(result.1);
    }
}

impl Default for RxDescriptor {
    fn default() -> Self {
        RxDescriptor {
            word0: 0,
            word1: 0,
        }
    }
}

impl DescriptorEntry for RxDescriptor {
    fn initialize(&mut self, address: *const u8) {
        self.modify(|w| w.set_address(address));
    }

    fn set_wrap(&mut self) {
        self.modify(|w| w.set_wrap());
    }
}

pub struct RxDescriptorReader(u32, u32);
impl RxDescriptorReader {
    pub fn is_wrap(&self) -> bool {
        self.0 & (1 << 0) != 0x0
    }

    pub fn is_owned(&self) -> bool {
        self.0 & (1 << 1) != 0x0
    }

    pub fn buffer_size(&self) -> u16 {
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
