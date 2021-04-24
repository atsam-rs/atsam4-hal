use volatile::*;
use super::descriptor_block::DescriptorEntry;

#[repr(C)]
#[derive(Clone, Copy, Default)]
pub struct RxDescriptor {
    pub word0: u32,
    pub word1: u32,
}

impl RxDescriptor {
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
}

impl DescriptorEntry for RxDescriptor {
    fn set_wrap(&mut self) {
        let mut word0 = Volatile::new(&mut self.word0);
        word0.update(|value| {
            *value |= 0x02;
        })
    }

    fn set_address(&mut self, address: *const u8) {
        if (address as u32) & 0x03 != 0 {
            panic!("Specified address is not 32 bit aligned");
        }

        let mut word0 = Volatile::new(&mut self.word0);
        word0.update(|value| {
            *value |= (address as u32) & !0x03;
        })
    }
}
