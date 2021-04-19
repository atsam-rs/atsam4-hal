pub struct Reader(u16);
impl Reader {
    pub(crate) fn new(bits: u16) -> Self {
        Self(bits)
    }

    pub fn bits(&self) -> u16 {
        self.0
    }
}
impl<FI> PartialEq<FI> for Reader
where
    FI: Copy + Into<u16>,
{
    fn eq(&self, other: &FI) -> bool {
        self.0.eq(&(*other).into())
    }
}

pub struct Writer(u16);
impl Writer {
    pub fn bits(&mut self, bits: u16) -> &mut Self {
        self.0 = bits;
        self
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

pub struct BMCR(u16);
impl BMCR {
    pub fn new() -> Self {
        BMCR(0)
    }

    pub fn modify<F>(&self, f: F)
    where
        for<'w> F: FnOnce(&Reader, &'w mut Writer) -> &'w mut Writer,
    {
        let bits = self.get();
        self.set(
            f(
                &Reader(bits),
                &mut Writer(bits),
            )
            .0,
        );
    }

    fn set(&mut self, value: u16) {
    }

    fn get(&self) -> u16 {
        0
    }
}
