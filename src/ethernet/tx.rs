use vcell::VolatileCell;
use super::descriptor_block::DescriptorEntry;

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

    pub fn set_byte_length(self, byte_length: u16) -> Self {
        if byte_length > 0x0000_1FFF {
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

    pub fn clear_wrap(self) -> Self {
        TxDescriptorWriter(self.0, self.1 & !(1 << 30))
    }

    pub fn set_used(self) -> Self {
        TxDescriptorWriter(self.0, self.1 | (1 << 31))
    }

    pub fn clear_used(self) -> Self {
        TxDescriptorWriter(self.0, self.1 & !(1 << 31))
    }
}
#[repr(C)]
pub struct TxDescriptor {
    pub word0: VolatileCell<u32>,
    pub word1: VolatileCell<u32>,
}

impl TxDescriptor {
    pub fn read(&self) -> TxDescriptorReader {
        TxDescriptorReader(self.word0.get(), self.word1.get())
    }

    pub fn modify<F: FnOnce(TxDescriptorWriter) -> TxDescriptorWriter>(&mut self, f: F) {
        let w = TxDescriptorWriter(self.word0.get(), self.word1.get());
        let result = f(w);
        self.word0.set(result.0);
        self.word1.set(result.1);
    }
}

impl DescriptorEntry for TxDescriptor {
    fn set_wrap(&mut self) {
        self.modify(|w| w.set_wrap())
    }

    fn set_address(&mut self, address: *const u8) {
        self.modify(|w| w.set_address(address))
    }
}
