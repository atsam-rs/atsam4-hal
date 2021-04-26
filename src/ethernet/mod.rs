use crate::{
    clock::{Enabled, GmacClock},
    pac::GMAC,
};
use core::marker::PhantomData;
use paste::paste;

mod descriptor_block;
mod eui48;
use eui48::Identifier as MacAddress;

mod phy;
mod rx;
mod tx;

#[cfg(feature = "smoltcp")]
mod smoltcp;

const MTU: usize = 1522;

trait VolatileReadWrite<T> {
    fn read_volatile(&self) -> T;
    fn write_volatile(&mut self, new_value: T);
}

impl VolatileReadWrite<u32> for u32 {
    fn read_volatile(&self) -> u32 {
        unsafe {
            core::ptr::read_volatile(self)
        }
    }

    fn write_volatile(&mut self, new_value: u32) {
        unsafe {
            core::ptr::write_volatile(self, new_value);
        }
    }
}

pub struct EthernetOptions {
    pub copy_all_frames: bool,
    pub disable_broadcast: bool,
    pub mac_address: MacAddress,
    pub alternate_mac_addresses: [Option<MacAddress>; 3],
}

impl Default for EthernetOptions {
    fn default() -> Self {
        EthernetOptions {
            copy_all_frames: false,
            disable_broadcast: false,
            alternate_mac_addresses: [None, None, None],
            mac_address: MacAddress::default(),
        }
    }
}

macro_rules! define_mac_address_function {
    (
        $address_number:expr
    ) => {
        paste! {
            fn [<set_mac_address $address_number>](&mut self, mac_address: &MacAddress) {
                let bytes = mac_address.as_bytes();
                self.gmac.[<sab $address_number>].write(|w| unsafe {
                    w.bits(
                        (bytes[0] as u32) |
                        (bytes[1] as u32) << 8 |
                        (bytes[2] as u32) << 16 |
                        (bytes[3] as u32) << 24
                    )
                });

                // NOTE: Writing the top bits (e.g. satX) enables the address in the hardware.
                self.gmac.[<sat $address_number>].write(|w| unsafe {
                    w.bits(
                        (bytes[4] as u32) |
                        (bytes[5] as u32) << 8
                    )
                });
            }
        }
    };
}

pub struct EthernetController {
    gmac: GMAC,
    clock: PhantomData<GmacClock<Enabled>>,
}

impl EthernetController {
    pub fn new(
        gmac: (GMAC, GmacClock<Enabled>), 
        options: EthernetOptions
    ) -> Self {
        let mut e = EthernetController {
            gmac: gmac.0,
            clock: PhantomData,
        };

        // Reset the GMAC to its reset state.
        e.reset();

        // Set the GMAC configuration register value.
        e.gmac.ncfgr.modify(|_, w| {
            w.
                // Don't write frame checksum bytes on received frames to memory.
                rfcs().set_bit().
                // Set pause-enable - transmission will pause if a non-zero 802.3 classic pause frame is received and PFC has not been negotiated.
                pen().set_bit();

            if options.copy_all_frames {
                w.caf().set_bit();
            }

            if options.disable_broadcast {
                w.nbc().set_bit();
            }

            w
        });

        // Set the MAC addresses into the hardware.
        e.set_mac_address1(&options.mac_address);
        for (i, x) in options.alternate_mac_addresses.iter().enumerate() {
            if let Some(alternate) = x {
                match i {
                    0 => e.set_mac_address2(alternate),
                    1 => e.set_mac_address3(alternate),
                    2 => e.set_mac_address4(alternate),
                    _ => panic!("unexpected alternate mac address offset in 3 element array"),
                }
            }
        }

        e
    }

    pub fn free(mut self) -> GMAC {
        self.reset();
        self.gmac
    }

    fn reset(&mut self) {
        self.gmac.ncr.reset();
        self.disable_all_interrupts();
        self.clear_statistics();

        // Clear all bits in the receive status register
        self.gmac.rsr.reset();

        // Clear all bits in the transmit status register
        self.gmac.tsr.reset();

        // Read the interrupt status register to ensure all interrupts are clear
        self.gmac.isr.read();

        // Reset the configuration register
        self.gmac.ncfgr.reset();
    }

    fn disable_all_interrupts(&mut self) {
        self.gmac.idr.write_with_zero(|w| {
            w.mfs()
                .set_bit()
                .rcomp()
                .set_bit()
                .rxubr()
                .set_bit()
                .txubr()
                .set_bit()
                .tur()
                .set_bit()
                .rlex()
                .set_bit()
                .tfc()
                .set_bit()
                .tcomp()
                .set_bit()
                .rovr()
                .set_bit()
                .hresp()
                .set_bit()
                .pfnz()
                .set_bit()
                .ptz()
                .set_bit()
                .pftr()
                .set_bit()
                .exint()
                .set_bit()
                .drqfr()
                .set_bit()
                .sfr()
                .set_bit()
                .drqft()
                .set_bit()
                .sft()
                .set_bit()
                .pdrqfr()
                .set_bit()
                .pdrsfr()
                .set_bit()
                .pdrqft()
                .set_bit()
                .pdrsft()
                .set_bit()
                .sri()
                .set_bit()
                .wol()
                .set_bit()
        });
    }

    fn enable_transmit(&mut self) {
        self.gmac.ncr.modify(|_, w| w.txen().set_bit())
    }

    fn disable_transmit(&mut self) {
        self.gmac.ncr.modify(|_, w| w.txen().clear_bit())
    }

    fn enable_receive(&mut self) {
        self.gmac.ncr.modify(|_, w| w.rxen().set_bit())
    }

    fn disable_receive(&mut self) {
        self.gmac.ncr.modify(|_, w| w.rxen().clear_bit())
    }

    // Hardware/MAC address manipulation
    define_mac_address_function!(1);
    define_mac_address_function!(2);
    define_mac_address_function!(3);
    define_mac_address_function!(4);

    // Statistics
    fn clear_statistics(&mut self) {
        self.gmac.ncr.modify(|_, w| w.clrstat().set_bit())
    }

    fn increment_statistics(&mut self) {
        self.gmac.ncr.modify(|_, w| w.incstat().set_bit())
    }

    // PHY interface
    fn is_phy_idle(&self) -> bool {
        self.gmac.nsr.read().idle().bit()
    }

    fn enable_management_port(&mut self) {
        self.gmac.ncr.modify(|_, w| w.mpe().set_bit())
    }

    fn disable_management_port(&mut self) {
        self.gmac.ncr.modify(|_, w| w.mpe().clear_bit())
    }
}
