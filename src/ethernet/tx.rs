use volatile::*;
use super::descriptor_block::DescriptorEntry;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct TxDescriptor {
    pub word0: u32,
    pub word1: u32,
}

impl TxDescriptor {
    pub fn set_address(&mut self, address: *const u32) {
        let mut word0 = Volatile::new(&mut self.word0);
        word0.write(address as u32);
    }
}

impl DescriptorEntry for TxDescriptor {
    fn set_wrap(&mut self) {
        let mut word1 = Volatile::new(&mut self.word0);
        word1.update(|value| {
            *value |= 1 << 15;
        })
    }

    fn set_address(&mut self, address: *const u8) {
        let mut word0 = Volatile::new(&mut self.word0);
        word0.write(address as u32);
    }
}
