use core::marker::PhantomData;
use crate::pac::gmac::MAN;
use crate::pac::GMAC;

pub struct Phy {
    man: PhantomData<MAN>,
}

impl Phy {
    pub fn new(man: MAN) -> Self {
        Phy {
            man: PhantomData,
        }
    }

    fn man(&mut self) -> &MAN {
        unsafe { &(*GMAC::ptr()).man }
    }
}
