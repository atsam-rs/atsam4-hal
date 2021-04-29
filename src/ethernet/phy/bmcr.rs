enum BitNumber {
    Speed1Gbps = 6,
    CollisionTest = 7,
    FullDuplex = 8,
    RestartAutoNegotiation = 9,
    Isolate = 10,
    PowerDown = 11,
    EnableAutoNegotiation = 12,
    Speed100Mbps = 13,
    LoopBack = 14,
    Reset = 15,
}

pub struct Bmcr(pub(super) u16);
impl Bmcr {
    pub fn new(initial_value: u16) -> Self {
        Bmcr(initial_value)
    }

    pub fn set_speed_1000(self) -> Self {
        Self(self.0 | (1 << BitNumber::Speed1Gbps as u32))
    }

    pub fn set_collision_test(self) -> Self {
        Self(self.0 | (1 << BitNumber::CollisionTest as u32))
    }

    pub fn set_full_duplex(self) -> Self {
        Self(self.0 | (1 << BitNumber::FullDuplex as u32))
    }

    pub fn set_auto_negotiation_restart(self) -> Self {
        Self(self.0 | (1 << BitNumber::RestartAutoNegotiation as u32))
    }

    pub fn set_isolate(self) -> Self {
        Self(self.0 | (1 << BitNumber::Isolate as u32))
    }

    pub fn set_power_down(self) -> Self {
        Self(self.0 | (1 << BitNumber::PowerDown as u32))
    }

    pub fn set_enable_auto_negotiation(self) -> Self {
        Self(self.0 | (1 << BitNumber::EnableAutoNegotiation as u32))
    }

    pub fn set_speed_100(self) -> Self {
        Self(self.0 | (1 << BitNumber::Speed100Mbps as u32))
    }

    pub fn set_loop_back(self) -> Self {
        Self(self.0 | (1 << BitNumber::LoopBack as u32))
    }

    pub fn set_reset(self) -> Self {
        Self(self.0 | (1 << BitNumber::Reset as u32))
    }
}
