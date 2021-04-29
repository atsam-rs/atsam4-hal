use embedded_time::rate::*;

mod bmsr;   // Basic Mode Status Register
use bmsr::*;

pub enum Register {
    Bmcr = 0x00,
    Bmsr = 0x01,
}

pub struct Writer(u16);
impl Writer {
    pub fn new(initial_value: u16) -> Self {
        Writer(initial_value)
    }

    pub fn set_speed_1000(self) -> Self {
        Self(self.0 | 0x0040)
    }

    pub fn set_collision_test(self) -> Self {
        Self(self.0 | 0x0080)
    }

    pub fn set_full_duplex(self) -> Self {
        Self(self.0 | 0x0100)
    }

    pub fn set_auto_negotiation_restart(self) -> Self {
        Self(self.0 | 0x0200)
    }

    pub fn set_isolate(self) -> Self {
        Self(self.0 | 0x0400)
    }

    pub fn set_power_down(self) -> Self {
        Self(self.0 | 0x0800)
    }

    pub fn set_enable_auto_negotiation(self) -> Self {
        Self(self.0 | 0x1000)
    }

    pub fn set_speed_100(self) -> Self {
        Self(self.0 | 0x2000)
    }

    pub fn set_loop_back(self) -> Self {
        Self(self.0 | 0x4000)
    }

    pub fn set_reset(self) -> Self {
        Self(self.0 | 0x8000)
    }
}

pub struct Status {
    pub(self) bmsr: Bmsr,
}

impl Status {
    pub fn link_detected(&self) -> bool {
        self.bmsr.link_detected()
    }

    pub fn speed(&self) -> MegabitsPerSecond {
        self.bmsr.speed()
    }

    pub fn is_full_duplex(&self) -> bool {
        self.bmsr.is_full_duplex()
    }
}

impl PartialEq for Status {
    fn eq(&self, other: &Status) -> bool {
        (!self.link_detected() && !other.link_detected())
            || (self.link_detected() == other.link_detected()
                && self.is_full_duplex() == other.is_full_duplex()
                && self.speed() == other.speed())
    }
}

pub trait Phy {
    fn modify<F: FnOnce(Writer) -> Writer>(&mut self, f: F) {
        self.enable_management_port();

        let w = Writer::new(self.read_register(Register::Bmcr));
        let new_value = f(w);
        self.write_register(Register::Bmcr, new_value.0);

        self.disable_management_port();
    } 

    fn read_register(&self, register: Register) -> u16;
    fn write_register(&mut self, register: Register, new_value: u16);
    fn wait_for_idle(&self);
    fn enable_management_port(&self);
    fn disable_management_port(&self);
    fn status(&self) -> Status {
        let bmsr = Bmsr::new(self.read_register(Register::Bmsr));
        Status {
            bmsr,
        }
    }
}
