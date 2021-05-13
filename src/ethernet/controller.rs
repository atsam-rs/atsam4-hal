use crate::{
    clock::{Enabled, GmacClock},
    pac::GMAC,
};
use super::{
    builder::Builder,
    eui48::Identifier as EthernetAddress,
    phy::{Phy, Register, Reader as PhyReader},
    Receiver, RxError,
    Transmitter, TxError,
};
use core::marker::PhantomData;
use paste::paste;

macro_rules! define_ethernet_address_function {
    (
        $address_number:expr
    ) => {
        paste! {
            fn [<set_ethernet_address $address_number>](&mut self, ethernet_address: &EthernetAddress) {
                let bytes = ethernet_address.as_bytes();
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

pub struct Controller<'rxtx, RX: Receiver, TX: Transmitter> {
    gmac: GMAC,
    clock: PhantomData<GmacClock<Enabled>>,
    rx: &'rxtx mut RX,
    tx: &'rxtx mut TX,
}

impl<'rxtx, RX: Receiver, TX: Transmitter> Controller<'rxtx, RX, TX> {
    pub(super) fn new(
        gmac: GMAC, _: GmacClock<Enabled>,
        rx: &'rxtx mut RX,
        tx: &'rxtx mut TX,
        builder: Builder) -> Self {

        let mut e = Controller {
            gmac,
            clock: PhantomData,
            rx,
            tx,
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
            w
        });

        // Set the MAC addresses into the hardware.
        e.set_ethernet_address1(&builder.ethernet_address());
        for index in 0..builder.alternate_ethernet_address_count() {
            let alternate_address = builder.alternate_ethernet_address(index);
            match index {
                0 => e.set_ethernet_address2(&alternate_address),
                1 => e.set_ethernet_address3(&alternate_address),
                2 => e.set_ethernet_address4(&alternate_address),
                _ => panic!("unexpected alternate mac address offset in 3 element array"),
            }
        }

        // Initialize the GMAC's DMA controller for each buffer descriptor table.
        // e.rx.setup_dma(&e.gmac);
        // e.tx.setup_dma(&e.gmac);

        // Enable receive and transmit circuits
        e.enable_receive();
        e.enable_transmit();

        // Set up interrupt handlers

        // Enable all interupts
        unimplemented!();

        e
    }

    pub fn status(&self) -> PhyReader {
        self.read()
    }

    fn send<F: FnOnce(&mut [u8], u16)>(&mut self, size: u16, f: F) -> Result<(), TxError> where Self: Sized {
        let result = self.tx.send(size, f);
//        self.tx_ring.demand_poll(&self.eth_dma);
        result
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
    define_ethernet_address_function!(1);
    define_ethernet_address_function!(2);
    define_ethernet_address_function!(3);
    define_ethernet_address_function!(4);

    // Statistics
    fn clear_statistics(&mut self) {
        self.gmac.ncr.modify(|_, w| w.clrstat().set_bit())
    }

    fn increment_statistics(&mut self) {
        self.gmac.ncr.modify(|_, w| w.incstat().set_bit())
    }
}

impl<'rxtx, RX: Receiver, TX: Transmitter> Phy for Controller<'rxtx, RX, TX> {
    fn read_register(&self, register: Register) -> u16 {
        self.wait_for_idle();
        self.gmac.man.modify(|r, w| unsafe { w.
            wtn().bits(0b10).                   // must always be binary 10 (0x02)
            rega().bits(register as u8).        // phy register to read
            phya().bits(0x0).                   // phy address
            op().bits(0b01).                    // read = 0b01, write = 0b10
            cltto().clear_bit().
            wzo().clear_bit()                   // must be set to zero
        });

        // Wait for the shift operation to complete and the register value to be present
        self.wait_for_idle();

        // Read the data portion of the register
        self.gmac.man.read().data().bits()
    }

    fn write_register(&mut self, register: Register, new_value: u16) {
        self.wait_for_idle();
        self.gmac.man.modify(|_, w| unsafe { w.
            data().bits(new_value).
            wtn().bits(0b10).                   // must always be binary 10 (0x02)
            rega().bits(register as u8).        // phy register to read
            phya().bits(0x0).                   // phy address
            op().bits(0b10).                    // read = 0b01, write = 0b10
            cltto().clear_bit().
            wzo().clear_bit()                   // must be set to zero
        });
        self.wait_for_idle();
    }

    fn wait_for_idle(&self) {
        while !self.gmac.nsr.read().idle().bit() {}
    }

    fn enable_management_port(&self) {
        self.gmac.ncr.modify(|_, w| w.mpe().set_bit());
    }

    fn disable_management_port(&self) {
        self.gmac.ncr.modify(|_, w| w.mpe().clear_bit());
    }
}