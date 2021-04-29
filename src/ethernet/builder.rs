use super::EthernetAddress;
use super::Controller;
use crate::{
    clock::{Enabled, GmacClock},
    pac::GMAC,
};

pub struct Builder {
    ethernet_address: EthernetAddress,
    alternate_addresses: [Option<EthernetAddress>; 3],
    alternate_address_count: usize,
    disable_broadcast: bool,
}

impl Builder {
    pub fn new() -> Self {
        Builder {
            ethernet_address: EthernetAddress::default(), 
            alternate_addresses: [None; 3],
            alternate_address_count: 0,
            disable_broadcast: false,
        }
    }

    pub fn set_ethernet_address(mut self, ethernet_address: EthernetAddress) -> Self {
        self.ethernet_address = ethernet_address;
        self
    }

    pub fn ethernet_address(&self) -> EthernetAddress {
        self.ethernet_address
    }

    pub fn add_alternate_ethernet_address(mut self, ethernet_address: EthernetAddress) -> Self {
        if self.alternate_address_count == 3 {
            panic!("Attempted to add more than three alternate addresses");
        }

        self.alternate_addresses[self.alternate_address_count] = Some(ethernet_address);
        self.alternate_address_count += 1;
        self
    }

    pub fn alternate_ethernet_address_count(&self) -> usize {
        self.alternate_address_count
    }

    pub fn alternate_ethernet_address(&self, index: usize) -> EthernetAddress {
        if index >= self.alternate_address_count {
            panic!("Attempted to access invalid alternate address");
        }

        self.alternate_addresses[index].unwrap()
    }

    pub fn disable_broadcast(mut self) -> Self {
        self.disable_broadcast = true;
        self
    }

    pub fn has_disable_broadcast(&self) -> bool {
        self.disable_broadcast
    }

    pub fn freeze(self, gmac: GMAC, clock: GmacClock<Enabled>) -> Controller {
        Controller::new(gmac, clock, self)
    }
}
