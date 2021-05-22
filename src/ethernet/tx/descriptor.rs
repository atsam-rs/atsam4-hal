use super::{VolatileReadWrite, MTU};

enum Word1BitNumbers {
    LastBuffer = 15,
    CRCNotAppended = 16,

    LateCollision = 26,
    FrameCorrupted = 27,
    Underrun = 28,
    RetryLimitExceeded = 29,
    Wrap = 30,
    Used = 31,
}

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TxDescriptor {
    // NOTE: Only read or write these fields using volatile operations
    word0: u32,
    word1: u32,
}

impl TxDescriptor {
    pub const fn const_default() -> Self {
        TxDescriptor { word0: 0, word1: 0 }
    }

    pub fn initialize(&mut self, address: *const u8) {
        self.write(|w| {
            w
            .set_used()
            .set_address(address)
            .set_buffer_size(0)
        })
    }

    pub fn read(&self) -> TxDescriptorReader {
        TxDescriptorReader(self.word0.read_volatile(), self.word1.read_volatile())
    }

    pub fn modify<F: FnOnce(TxDescriptorWriter) -> TxDescriptorWriter>(&mut self, f: F) {
        let w = TxDescriptorWriter(self.word0.read_volatile(), self.word1.read_volatile());
        let result = f(w);
        self.word0.write_volatile(result.0);
        self.word1.write_volatile(result.1);
    }

    pub fn write<F: FnOnce(TxDescriptorWriter) -> TxDescriptorWriter>(&mut self, f: F) {
        let w = TxDescriptorWriter(0, 0);
        let result = f(w);
        self.word0.write_volatile(result.0);
        self.word1.write_volatile(result.1);
    }

    fn set_wrap(&mut self) {
        self.modify(|w| w.set_wrap())
    }
}

pub struct TxDescriptorReader(u32, u32);
impl TxDescriptorReader {
    pub fn collided(&self) -> bool {
        self.1 & (1 << Word1BitNumbers::LateCollision as u32) != 0
    }

    pub fn corrupted(&self) -> bool {
        self.1 & (1 << Word1BitNumbers::FrameCorrupted as u32) != 0
    }

    pub fn underran(&self) -> bool {
        self.1 & (1 << Word1BitNumbers::Underrun as u32) != 0
    }

    pub fn retry_exceeded(&self) -> bool {
        self.1 & (1 << Word1BitNumbers::RetryLimitExceeded as u32) != 0
    }

    pub fn is_used(&self) -> bool {
        self.1 & (1 << Word1BitNumbers::Used as u32) != 0
    }
}

pub struct TxDescriptorWriter(u32, u32);
impl TxDescriptorWriter {
    pub fn set_address(self, address: *const u8) -> Self {
        TxDescriptorWriter(address as u32, self.1)
    }

    pub fn set_buffer_size(self, byte_length: u16) -> Self {
        if byte_length as usize > MTU {
            panic!("Specified byte length is larger than 0x1FFFF");
        }
        TxDescriptorWriter(self.0, (self.1 & !0x0000_1FFF) | byte_length as u32)
    }

    pub fn set_wrap(self) -> Self {
        TxDescriptorWriter(self.0, self.1 | (1 << Word1BitNumbers::Wrap as u32))
    }

    pub fn set_used(self) -> Self {
        TxDescriptorWriter(self.0, self.1 | (1 << Word1BitNumbers::Used as u32))
    }

    pub fn clear_used(self) -> Self {
        TxDescriptorWriter(self.0, self.1 & !(1 << Word1BitNumbers::Used as u32))
    }
}
