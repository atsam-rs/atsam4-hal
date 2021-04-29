use embedded_time::rate::*;

mod bmcr;   // Basic Mode Control Register
pub(super) use bmcr::Bmcr as Writer;

mod bmsr;   // Basic Mode Status Register
pub(super) use bmsr::Bmsr as Reader;

pub enum Register {
    Bmcr = 0x00,
    Bmsr = 0x01,
}

pub trait Phy {
    fn modify<F: FnOnce(Writer) -> Writer>(&mut self, f: F) {
        self.enable_management_port();

        let w = Writer::new(self.read_register(Register::Bmcr));
        let new_value = f(w);
        self.write_register(Register::Bmcr, new_value.0);

        self.disable_management_port();
    }

    fn read(&self) -> Reader {
        Reader::new(self.read_register(Register::Bmsr))
    }

    fn read_register(&self, register: Register) -> u16;
    fn write_register(&mut self, register: Register, new_value: u16);
    fn wait_for_idle(&self);
    fn enable_management_port(&self);
    fn disable_management_port(&self);
}
