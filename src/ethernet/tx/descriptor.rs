use super::{
    MTU,
    VolatileReadWrite,
};

#[repr(C)]
#[derive(Clone, Copy)]
pub struct TxDescriptor {
    // NOTE: Only read or write these fields using volatile operations
    word0: u32,
    word1: u32,
}

impl TxDescriptor {
    pub const fn const_default() -> Self {
        TxDescriptor {
            word0: 0,
            word1: 0,
        }        
    }

    pub fn initialize(&mut self, address: *const u8) {
        self.modify(|w| w
            .clear_used()
            .clear_end_of_frame()
            .set_address(address)
            .set_buffer_size(0)
        )
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

    fn set_wrap(&mut self) {
        self.modify(|w| w.set_wrap())
    }
}

pub struct TxDescriptorReader(u32, u32);
impl TxDescriptorReader {
    pub fn has_collision(&self) -> bool {
        self.1 & (1 << 26) != 0
    }

    pub fn has_corruption(&self) -> bool {
        self.1 & (1 << 27) != 0
    }

    pub fn has_underrun(&self) -> bool {
        self.1 & (1 << 28) != 0
    }

    pub fn has_retry_exceeded(&self) -> bool {
        self.1 & (1 << 29) != 0
    }

    pub fn is_wrap(&self) -> bool {
        self.1 & (1 << 30) != 0
    }

    pub fn is_used(&self) -> bool {
        self.1 & (1 << 31) != 0
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

    pub fn set_end_of_frame(self) -> Self {
        TxDescriptorWriter(self.0, self.1 | (1 << 15))
    }

    pub fn clear_end_of_frame(self) -> Self {
        TxDescriptorWriter(self.0, self.1 & !(1 << 15))
    }

    pub fn set_wrap(self) -> Self {
        TxDescriptorWriter(self.0, self.1 | (1 << 30))
    }

    pub fn set_used(self) -> Self {
        TxDescriptorWriter(self.0, self.1 | (1 << 31))
    }

    pub fn clear_used(self) -> Self {
        TxDescriptorWriter(self.0, self.1 & !(1 << 31))
    }
}
