use embedded_time::rate::*;

enum BitNumbers {
    ExtendedCapability = 0,
    JabberDetected = 1,
    LinkDetected = 2,
    AutoNegotiationCapable = 3,
    RemoteFaultDetected = 4,
    AutoNegotiationComplete = 5,
    PreambleSuppressionCapable = 6,
    HalfDuplex10BaseTCapable = 11,
    FullDuplex10BaseTCapable = 12,
    HalfDuplex100BaseTXCapable = 13,
    FullDuplex100BaseTXCapable = 14,
}

pub struct Bmsr(u16);
impl Bmsr {
    pub fn new(initial_value: u16) -> Self {
        Bmsr(initial_value)
    }

    pub fn has_extended_capability(&self) -> bool {
        self.0 & (1 << BitNumbers::ExtendedCapability as u32) != 0
    }

    pub fn jabber_detected(&self) -> bool {
        self.0 & (1 << BitNumbers::JabberDetected as u32) != 0
    }

    pub fn link_detected(&self) -> bool {
        self.0 & (1 << BitNumbers::LinkDetected as u32) != 0
    }

    pub fn auto_negotiation_capable(&self) -> bool {
        self.0 & (1 << BitNumbers::AutoNegotiationCapable as u32) != 0
    }

    pub fn remote_fault_detected(&self) -> bool {
        self.0 & (1 << BitNumbers::RemoteFaultDetected as u32) != 0
    }

    pub fn auto_negotiation_complete(&self) -> bool {
        self.0 & (1 << BitNumbers::AutoNegotiationComplete as u32) != 0
    }

    pub fn preamble_suppression_capable(&self) -> bool {
        self.0 & (1 << BitNumbers::PreambleSuppressionCapable as u32) != 0
    }

    pub fn half_duplex_10base_t_capable(&self) -> bool {
        self.0 & (1 << BitNumbers::HalfDuplex10BaseTCapable as u32) != 0
    }

    pub fn full_duplex_10base_t_capable(&self) -> bool {
        self.0 & (1 << BitNumbers::FullDuplex10BaseTCapable as u32) != 0
    }

    pub fn half_duplex_100base_tx_capable(&self) -> bool {
        self.0 & (1 << BitNumbers::HalfDuplex100BaseTXCapable as u32) != 0
    }

    pub fn full_duplex_100base_tx_capable(&self) -> bool {
        self.0 & (1 << BitNumbers::FullDuplex100BaseTXCapable as u32) != 0
    }

    pub fn is_full_duplex(&self) -> bool {
        (self.0 & (1 << BitNumbers::FullDuplex10BaseTCapable as u32)
                & (1 << BitNumbers::FullDuplex100BaseTXCapable as u32)) != 0
    }

    pub fn is_10mbit(&self) -> bool {
        (self.0 & (1 << BitNumbers::HalfDuplex10BaseTCapable as u32)
                & (1 << BitNumbers::FullDuplex10BaseTCapable as u32)) != 0
    }

    pub fn is_100mbit(&self) -> bool {
        (self.0 & (1 << BitNumbers::HalfDuplex100BaseTXCapable as u32)
                & (1 << BitNumbers::FullDuplex100BaseTXCapable as u32)) != 0
    }

    pub fn speed(&self) -> MegabitsPerSecond {
        if self.is_100mbit() {
            MegabitsPerSecond(10)
        } else if self.is_100mbit() {
            MegabitsPerSecond(100)
        } else {
            MegabitsPerSecond(0)
        }
    }
}
