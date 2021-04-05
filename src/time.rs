// NOTE: This file was copied from the atsamd-rs project: https://github.com/atsamd-rs/atsamd/blob/master/hal/src/common/time.rs

//! Time units

// Frequency based

/// Bits per second
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Bps(pub u32);

/// Hertz
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Hertz(pub u32);

/// KiloHertz
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct KiloHertz(pub u32);

/// MegaHertz
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MegaHertz(pub u32);

// Period based

/// Seconds
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Seconds(pub u32);

/// Miliseconds
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Miliseconds(pub u32);

/// Microseconds
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Microseconds(pub u32);

/// Extension trait that adds convenience methods to the `u32` type
pub trait U32Ext {
    /// Wrap in `Bps`
    fn bps(self) -> Bps;

    /// Wrap in `Hertz`
    fn hz(self) -> Hertz;

    /// Wrap in `KiloHertz`
    fn khz(self) -> KiloHertz;

    /// Wrap in `MegaHertz`
    fn mhz(self) -> MegaHertz;

    /// Wrap in `Seconds`
    fn s(self) -> Seconds;

    /// Wrap in `Miliseconds`
    fn ms(self) -> Miliseconds;

    /// Wrap in `Microseconds`
    fn us(self) -> Microseconds;
}

impl U32Ext for u32 {
    // Frequency based

    fn bps(self) -> Bps {
        Bps(self)
    }

    fn hz(self) -> Hertz {
        Hertz(self)
    }

    fn khz(self) -> KiloHertz {
        KiloHertz(self)
    }

    fn mhz(self) -> MegaHertz {
        MegaHertz(self)
    }

    // Period based

    fn s(self) -> Seconds {
        Seconds(self)
    }

    fn ms(self) -> Miliseconds {
        Miliseconds(self)
    }

    fn us(self) -> Microseconds {
        Microseconds(self)
    }
}

// Frequency based

impl From<KiloHertz> for Hertz {
    fn from(f: KiloHertz) -> Hertz {
        Hertz(f.0 * 1_000)
    }
}

impl From<MegaHertz> for Hertz {
    fn from(f: MegaHertz) -> Hertz {
        Hertz(f.0 * 1_000_000)
    }
}

impl From<MegaHertz> for KiloHertz {
    fn from(f: MegaHertz) -> KiloHertz {
        KiloHertz(f.0 * 1_000)
    }
}

impl From<Hertz> for KiloHertz {
    fn from(f: Hertz) -> KiloHertz {
        KiloHertz(f.0 / 1_000)
    }
}

impl From<Hertz> for MegaHertz {
    fn from(f: Hertz) -> MegaHertz {
        MegaHertz(f.0 / 1_000_000)
    }
}

impl From<KiloHertz> for MegaHertz {
    fn from(f: KiloHertz) -> MegaHertz {
        MegaHertz(f.0 / 1_000)
    }
}

// Period based

impl From<Seconds> for Miliseconds {
    fn from(s: Seconds) -> Miliseconds {
        Miliseconds(s.0 * 1_000)
    }
}

impl From<Seconds> for Microseconds {
    fn from(s: Seconds) -> Microseconds {
        Microseconds(s.0 * 1_000_000)
    }
}

impl From<Miliseconds> for Microseconds {
    fn from(ms: Miliseconds) -> Microseconds {
        Microseconds(ms.0 * 1_000)
    }
}

impl From<Miliseconds> for Seconds {
    fn from(ms: Miliseconds) -> Seconds {
        Seconds(ms.0 / 1_000)
    }
}

impl From<Microseconds> for Seconds {
    fn from(us: Microseconds) -> Seconds {
        Seconds(us.0 / 1_000_000)
    }
}

impl From<Microseconds> for Miliseconds {
    fn from(us: Microseconds) -> Miliseconds {
        Miliseconds(us.0 / 1_000)
    }
}

impl From<Bps> for Hertz {
    fn from(bps: Bps) -> Hertz {
        Hertz(bps.0)
    }
}

// Frequency <-> Period

impl From<Microseconds> for Hertz {
    fn from(us: Microseconds) -> Hertz {
        Hertz(1_000_000_u32 / us.0)
    }
}

impl From<Hertz> for Microseconds {
    fn from(f: Hertz) -> Microseconds {
        Microseconds(1_000_000_u32 / f.0)
    }
}

#[cfg(test)]
mod tests {
    use crate::time::*;

    #[test]
    fn convert_us_to_hz() {
        let as_us: Microseconds = 3.hz().into();
        assert_eq!(as_us.0, 333_333_u32);
    }

    #[test]
    fn convert_ms_to_us() {
        let as_us: Microseconds = 3.ms().into();
        assert_eq!(as_us.0, 3_000_u32);
    }

    #[test]
    fn convert_mhz_to_hz() {
        let as_hz: Hertz = 48.mhz().into();
        assert_eq!(as_hz.0, 48_000_000_u32);
    }
}
