pub trait VolatileReadWrite {
    fn read_volatile(&self) -> u32;
    fn write_volatile(&mut self, new_value: u32);
}

impl VolatileReadWrite for u32 {
    fn read_volatile(&self) -> u32 {
        unsafe { core::ptr::read_volatile(self) }
    }
    
    fn write_volatile(&mut self, new_value: u32) {
        unsafe {
            core::ptr::write_volatile(self, new_value);
        }
    }
}
