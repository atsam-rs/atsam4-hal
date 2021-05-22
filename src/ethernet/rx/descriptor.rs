use super::VolatileReadWrite;

enum Word0BitNumbers {
    Owned = 0,
    Wrap = 1,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct RxDescriptor {
    // NOTE: Only read or write these fields using volatile operations
    word0: u32,
    word1: u32,
}

impl RxDescriptor {
    pub const fn const_default() -> Self {
        RxDescriptor { word0: 0, word1: 0 }
    }

    pub fn initialize(&mut self, address: *const u8) {
        self.write(|w| {
            w
            .set_address(address)
            .clear_owned()
        })
    }

    pub fn read(&self) -> RxDescriptorReader {
        RxDescriptorReader(self.word0.read_volatile(), self.word1.read_volatile())
    }

    pub fn modify<F: FnOnce(RxDescriptorWriter) -> RxDescriptorWriter>(&mut self, f: F) {
        let w = RxDescriptorWriter(self.word0.read_volatile(), self.word1.read_volatile());
        let result = f(w);
        self.word0.write_volatile(result.0);
        self.word1.write_volatile(result.1);
    }

    pub fn write<F: FnOnce(RxDescriptorWriter) -> RxDescriptorWriter>(&mut self, f: F) {
        let w = RxDescriptorWriter(0, 0);
        let result = f(w);
        self.word0.write_volatile(result.0);
        self.word1.write_volatile(result.1);
    }
}

pub struct RxDescriptorReader(u32, u32);
impl RxDescriptorReader {
    pub fn is_owned(&self) -> bool {
        self.0 & (1 << Word0BitNumbers::Owned as u32) != 0x0
    }

    pub fn buffer_size(&self) -> u16 {
        //!todo - If jumbo frames are enabled, this needs to take into account the 13th bit as well.
        (self.1 & 0x0000_0FFF) as u16
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

    pub fn clear_owned(self) -> Self {
        RxDescriptorWriter(self.0 & !(1 << Word0BitNumbers::Owned as u32), self.1)
    }

    pub fn set_wrap(self) -> Self {
        RxDescriptorWriter(self.0 | (1 << Word0BitNumbers::Wrap as u32), self.1)
    }
}
