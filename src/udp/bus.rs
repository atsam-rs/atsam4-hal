use crate::clock::{Disabled, UdpClock};
use crate::gpio::{Pb10, Pb11, SysFn};
use crate::pac::{PMC, UDP};
use crate::udp::{frm_num, Endpoint, UdpEndpointAddress, UdpEndpointType, UdpUsbDirection};
use crate::BorrowUnchecked;
use core::cell::RefCell;
use core::marker::PhantomData;
use cortex_m::interrupt::Mutex;
use usb_device::{
    bus::{PollResult, UsbBus},
    endpoint::{EndpointAddress, EndpointType},
    UsbDirection,
};

#[cfg(feature = "atsam4s")]
use crate::clock::{disable_pllb_clock, reenable_pllb_clock, wait_for_pllb_lock};

pub const NUM_ENDPOINTS: usize = 8;

pub struct UdpBus {
    udp: Mutex<RefCell<UDP>>,
    endpoints: [Mutex<RefCell<Endpoint>>; NUM_ENDPOINTS],
    clock: PhantomData<UdpClock<Disabled>>,
    ddm: PhantomData<Pb10<SysFn>>,
    ddp: PhantomData<Pb11<SysFn>>,
    sof_errors: Mutex<RefCell<u32>>,
}

impl UdpBus {
    /// Initialize UDP as a USB device
    pub fn new(udp: UDP, _clock: UdpClock<Disabled>, _ddm: Pb10<SysFn>, _ddp: Pb11<SysFn>) -> Self {
        let endpoints = [
            Mutex::new(RefCell::new(Endpoint::new(0))),
            Mutex::new(RefCell::new(Endpoint::new(1))),
            Mutex::new(RefCell::new(Endpoint::new(2))),
            Mutex::new(RefCell::new(Endpoint::new(3))),
            Mutex::new(RefCell::new(Endpoint::new(4))),
            Mutex::new(RefCell::new(Endpoint::new(5))),
            Mutex::new(RefCell::new(Endpoint::new(6))),
            Mutex::new(RefCell::new(Endpoint::new(7))),
        ];
        let udp = Mutex::new(RefCell::new(udp));
        let sof_errors = Mutex::new(RefCell::new(0));
        Self {
            udp,
            endpoints,
            clock: PhantomData,
            ddm: PhantomData,
            ddp: PhantomData,
            sof_errors,
        }
    }

    /// Enables UDP MCK (from MCK)
    fn enable_periph_clk(&self) {
        #[cfg(feature = "atsam4e")]
        PMC::borrow_unchecked(|pmc| pmc.pmc_pcer1.write_with_zero(|w| w.pid35().set_bit()));
        #[cfg(feature = "atsam4s")]
        PMC::borrow_unchecked(|pmc| pmc.pmc_pcer1.write_with_zero(|w| w.pid34().set_bit()));
    }

    /// Disables UDP MCK (from MCK)
    /// Used when entering USB suspend state
    fn disable_periph_clk(&self) {
        #[cfg(feature = "atsam4e")]
        PMC::borrow_unchecked(|pmc| pmc.pmc_pcdr1.write_with_zero(|w| w.pid35().set_bit()));
        #[cfg(feature = "atsam4s")]
        PMC::borrow_unchecked(|pmc| pmc.pmc_pcdr1.write_with_zero(|w| w.pid34().set_bit()));
    }

    /// Disables each of the endpoints
    /// Also flushes resets/flushes the fifo
    fn disable(&self) {
        // Enable UDP MCK (from MCK)
        self.enable_periph_clk();

        cortex_m::interrupt::free(|cs| {
            // Disable endpoints
            for i in 0..NUM_ENDPOINTS {
                self.endpoints[i].borrow(cs).borrow_mut().disable();
            }

            // Disable Transceiver (TXDIS)
            // Disable 1.5k pullup
            self.udp
                .borrow(cs)
                .borrow()
                .txvc
                .modify(|_, w| w.txvdis().set_bit().puon().clear_bit());
        });
    }

    /// Enable each of the configured endpoints
    /// Only allocated endpoints are enabled
    fn _enable(&self) {
        defmt::trace!("UdpBus::enable()");

        // Start with integrated 1.5k pull-up on D+ disabled
        cortex_m::interrupt::free(|cs| {
            self.udp
                .borrow(cs)
                .borrow()
                .txvc
                .modify(|_, w| w.puon().clear_bit());
        });

        // Enable UDP MCK (from MCK)
        self.enable_periph_clk();

        // Enable fast restart signal
        PMC::borrow_unchecked(|pmc| pmc.pmc_fsmr.modify(|_, w| w.usbal().set_bit()));

        // Enable UDP Clock (UDPCK)
        PMC::borrow_unchecked(|pmc| pmc.pmc_scer.write_with_zero(|w| w.udp().set_bit()));

        // Enable integrated 1.5k pull-up on D+
        cortex_m::interrupt::free(|cs| {
            self.udp
                .borrow(cs)
                .borrow()
                .txvc
                .modify(|_, w| w.puon().set_bit());

            // Enable allocated endpoints
            for i in 0..NUM_ENDPOINTS {
                self.endpoints[i].borrow(cs).borrow_mut().clear_fifo();
                self.endpoints[i].borrow(cs).borrow_mut().enable();
            }
        });
    }
}

impl UsbBus for UdpBus {
    fn alloc_ep(
        &mut self,
        ep_dir: UsbDirection,
        ep_addr: Option<EndpointAddress>,
        ep_type: EndpointType,
        max_packet_size: u16,
        interval: u8,
    ) -> usb_device::Result<EndpointAddress> {
        defmt::trace!(
            "UdpBus::alloc_ep({:?}, {:?}, {:?}, {}, {})",
            UdpUsbDirection { inner: ep_dir },
            UdpEndpointAddress { inner: ep_addr },
            UdpEndpointType { inner: ep_type },
            max_packet_size,
            interval
        );
        cortex_m::interrupt::free(|cs| {
            match ep_addr {
                Some(ep_addr) => self.endpoints[ep_addr.index()]
                    .borrow(cs)
                    .borrow_mut()
                    .alloc(ep_type, ep_dir, max_packet_size, interval),
                None => {
                    // Iterate over all of the endpoints and try to allocate one
                    // Keep trying even if the first selection fails as there are different
                    // endpoint specs for each one.
                    // Only Control OUT endpoints are allocated, Control Endpoints are shared between
                    // IN and OUT (allocated a Control IN endpoint is a no-op).
                    for i in 0..NUM_ENDPOINTS {
                        match self.endpoints[i].borrow(cs).borrow_mut().alloc(
                            ep_type,
                            ep_dir,
                            max_packet_size,
                            interval,
                        ) {
                            Ok(addr) => {
                                return Ok(addr);
                            }
                            Err(usb_device::UsbError::Unsupported) => {} // Invalid configuration try next
                            Err(usb_device::UsbError::InvalidEndpoint) => {} // Already allocated
                            Err(usb_device::UsbError::EndpointMemoryOverflow) => {} // Max packet too large
                            Err(_) => return Err(usb_device::UsbError::Unsupported),
                        }
                    }

                    // Couldn't find a free endpoint as specified
                    Err(usb_device::UsbError::InvalidEndpoint)
                }
            }
        })
    }

    /// Enable each of the configured endpoints
    /// Only allocated endpoints are enabled
    fn enable(&mut self) {
        self._enable();
    }

    /// Resets state of all endpoints and peripheral flags so that they can be enumerated
    /// Clears each of the fifos and configured state of the device.
    fn reset(&self) {
        let txvc_reg = UDP::borrow_unchecked(|udp| udp.txvc.as_ptr());
        let imr_reg = UDP::borrow_unchecked(|udp| udp.imr.as_ptr());
        let faddr_reg = UDP::borrow_unchecked(|udp| udp.faddr.as_ptr());
        let glb_stat_reg = UDP::borrow_unchecked(|udp| udp.glb_stat.as_ptr());
        defmt::trace!(
            "{} UdpBus::reset() txvc:{:#x} imr:{:#x} faddr:{:#x} glb_stat:{:#x}",
            frm_num(),
            unsafe { core::ptr::read(txvc_reg) },
            unsafe { core::ptr::read(imr_reg) },
            unsafe { core::ptr::read(faddr_reg) },
            unsafe { core::ptr::read(glb_stat_reg) }
        );

        cortex_m::interrupt::free(|cs| {
            // Enable transceiver
            self.udp
                .borrow(cs)
                .borrow()
                .txvc
                .modify(|_, w| w.txvdis().clear_bit());

            // Disable address and configured state
            // Make sure remote wakeup is enabled
            self.udp.borrow(cs).borrow().glb_stat.modify(|_, w| {
                w.confg()
                    .clear_bit()
                    .fadden()
                    .clear_bit()
                    .rmwupe()
                    .set_bit()
            });

            // Set Device Address to 0 and enable FEN
            self.udp
                .borrow(cs)
                .borrow()
                .faddr
                .modify(|_, w| unsafe { w.fen().set_bit().fadd().bits(0) });

            // Enable general UDP interrupts
            self.udp
                .borrow(cs)
                .borrow()
                .ier
                .write_with_zero(|w| w.rxsusp().set_bit().sofint().set_bit());
        });

        // Reset endpoints
        for i in 0..NUM_ENDPOINTS {
            cortex_m::interrupt::free(|cs| {
                self.endpoints[i].borrow(cs).borrow_mut().reset();
            });
        }

        let txvc_reg = UDP::borrow_unchecked(|udp| udp.txvc.as_ptr());
        let imr_reg = UDP::borrow_unchecked(|udp| udp.imr.as_ptr());
        let faddr_reg = UDP::borrow_unchecked(|udp| udp.faddr.as_ptr());
        let glb_stat_reg = UDP::borrow_unchecked(|udp| udp.glb_stat.as_ptr());
        defmt::trace!(
            "{} UdpBus::reset() (Updated) txvc:{:#x} imr:{:#x} faddr:{:#x} glb_stat:{:#x}",
            frm_num(),
            unsafe { core::ptr::read(txvc_reg) },
            unsafe { core::ptr::read(imr_reg) },
            unsafe { core::ptr::read(faddr_reg) },
            unsafe { core::ptr::read(glb_stat_reg) }
        );
    }

    /// Sets the device address, FEN (Function Enabled) and FADDEN (Function Address Enable)
    fn set_device_address(&self, addr: u8) {
        defmt::info!("{} UdpBus::set_device_address({})", frm_num(), addr);
        cortex_m::interrupt::free(|cs| {
            // Set Device Address and FEN
            self.udp
                .borrow(cs)
                .borrow()
                .faddr
                .modify(|_, w| unsafe { w.fen().set_bit().fadd().bits(addr) });

            // Set FADDEN
            self.udp
                .borrow(cs)
                .borrow()
                .glb_stat
                .modify(|_, w| w.fadden().set_bit());
        });
    }

    fn write(&self, ep_addr: EndpointAddress, buf: &[u8]) -> usb_device::Result<usize> {
        defmt::trace!(
            "{} UdpBus::write({:?}, {:02X})",
            frm_num(),
            UdpEndpointAddress {
                inner: Some(ep_addr)
            },
            buf
        );
        cortex_m::interrupt::free(|cs| {
            // Make sure the endpoint is configured correctly
            if self.endpoints[ep_addr.index()]
                .borrow(cs)
                .borrow()
                .address()
                .index()
                != ep_addr.index()
            {
                return Err(usb_device::UsbError::InvalidEndpoint);
            }

            self.endpoints[ep_addr.index()]
                .borrow(cs)
                .borrow_mut()
                .write(buf)
        })
    }

    fn read(&self, ep_addr: EndpointAddress, buf: &mut [u8]) -> usb_device::Result<usize> {
        defmt::trace!(
            "{} UdpBus::read({:02X})",
            frm_num(),
            UdpEndpointAddress {
                inner: Some(ep_addr)
            },
        );
        cortex_m::interrupt::free(|cs| {
            // Make sure the endpoint is configured correctly
            if self.endpoints[ep_addr.index()]
                .borrow(cs)
                .borrow()
                .address()
                .index()
                != ep_addr.index()
            {
                return Err(usb_device::UsbError::InvalidEndpoint);
            }

            self.endpoints[ep_addr.index()]
                .borrow(cs)
                .borrow_mut()
                .read(buf)
        })
    }

    fn set_stalled(&self, ep_addr: EndpointAddress, stalled: bool) {
        cortex_m::interrupt::free(|cs| {
            if stalled {
                self.endpoints[ep_addr.index()]
                    .borrow(cs)
                    .borrow_mut()
                    .stall();
            } else {
                self.endpoints[ep_addr.index()]
                    .borrow(cs)
                    .borrow_mut()
                    .unstall();
            }
        });
    }

    fn is_stalled(&self, ep_addr: EndpointAddress) -> bool {
        cortex_m::interrupt::free(|cs| {
            self.endpoints[ep_addr.index()]
                .borrow(cs)
                .borrow()
                .is_stalled()
        })
    }

    fn suspend(&self) {
        defmt::trace!("{} UdpBus::suspend()", frm_num());
        // Disable Transceiver
        cortex_m::interrupt::free(|cs| {
            self.udp
                .borrow(cs)
                .borrow()
                .txvc
                .modify(|_, w| w.txvdis().set_bit());
        });

        // Disable UDP MCK (from MCK)
        #[cfg(feature = "atsam4e")]
        PMC::borrow_unchecked(|pmc| pmc.pmc_pcdr1.write_with_zero(|w| w.pid35().set_bit()));
        #[cfg(feature = "atsam4s")]
        PMC::borrow_unchecked(|pmc| pmc.pmc_pcdr1.write_with_zero(|w| w.pid34().set_bit()));

        // Disable UDPCK (from PLL)
        PMC::borrow_unchecked(|pmc| pmc.pmc_scer.write_with_zero(|w| w.udp().clear_bit()));

        // Disable PLLB (atsam4s only)
        #[cfg(feature = "atsam4s")]
        PMC::borrow_unchecked(|pmc| disable_pllb_clock(pmc));
    }

    fn resume(&self) {
        defmt::trace!("{} UdpBus::resume()", frm_num());
        // Enable PLLB (atsam4s only)
        #[cfg(feature = "atsam4s")]
        PMC::borrow_unchecked(|pmc| {
            reenable_pllb_clock(pmc);
            wait_for_pllb_lock(pmc);
        });

        // Enable UDPCK (from PLL)
        PMC::borrow_unchecked(|pmc| pmc.pmc_scer.write_with_zero(|w| w.udp().set_bit()));

        // Enable UDP MCK (from MCK)
        #[cfg(feature = "atsam4e")]
        PMC::borrow_unchecked(|pmc| pmc.pmc_pcer1.write_with_zero(|w| w.pid35().set_bit()));
        #[cfg(feature = "atsam4s")]
        PMC::borrow_unchecked(|pmc| pmc.pmc_pcer1.write_with_zero(|w| w.pid34().set_bit()));

        // Enable Transceiver
        cortex_m::interrupt::free(|cs| {
            self.udp
                .borrow(cs)
                .borrow()
                .txvc
                .modify(|_, w| w.txvdis().clear_bit());
        });
    }

    fn poll(&self) -> PollResult {
        // UDP MCK must be enabled before reading/writing any UDP registers
        self.enable_periph_clk();

        // Read interrupt enabled status
        let imr = cortex_m::interrupt::free(|cs| self.udp.borrow(cs).borrow().imr.read());
        // Read interrupt status
        let isr = cortex_m::interrupt::free(|cs| self.udp.borrow(cs).borrow().isr.read());

        // Process SOF interrupt
        if imr.sofint().bit() && isr.sofint().bit() {
            cortex_m::interrupt::free(|cs| {
                // Clear SOF interrupt
                self.udp
                    .borrow(cs)
                    .borrow()
                    .icr
                    .write_with_zero(|w| w.sofint().set_bit());

                // Check for sof_eop (Start of Frame End of Packet) errors
                if self.udp.borrow(cs).borrow().frm_num.read().frm_err().bit() {
                    *self.sof_errors.borrow(cs).borrow_mut() += 1;
                }
            });
            return PollResult::None;
        }

        // Process endpoints - Return as soon as a pending operation is found
        let mut ep_out_result = 0;
        let mut ep_in_complete_result = 0;
        let mut ep_setup_result = 0;
        for i in 0..NUM_ENDPOINTS {
            let result = cortex_m::interrupt::free(|cs| {
                // Continue onto the next endpoint if no events found
                self.endpoints[i].borrow(cs).borrow_mut().poll()
            });
            // Accumulate status from each endpoint
            if let PollResult::Data {
                ep_out,
                ep_in_complete,
                ep_setup,
            } = result
            {
                ep_out_result |= ep_out;
                ep_in_complete_result |= ep_in_complete;
                ep_setup_result |= ep_setup;

                // Exit early if this is EP0
                if i == 0 {
                    break;
                }
            }
        }

        // Check if there's been a data event
        if (ep_out_result | ep_in_complete_result | ep_setup_result) > 0 {
            return PollResult::Data {
                ep_out: ep_out_result,
                ep_in_complete: ep_in_complete_result,
                ep_setup: ep_setup_result,
            };
        }

        // Process wakeup interrupt (wakeup or resume or external resume)
        if imr.wakeup().bit() && isr.wakeup().bit()
            || imr.rxrsm().bit() && isr.rxrsm().bit()
            || imr.extrsm().bit() && isr.extrsm().bit()
        {
            cortex_m::interrupt::free(|cs| {
                // Clear wakeup/resume interrputs
                self.udp
                    .borrow(cs)
                    .borrow()
                    .icr
                    .write_with_zero(|w| w.wakeup().set_bit().rxrsm().set_bit().extrsm().set_bit());

                // Disable wakeup/resume interrputs
                self.udp.borrow(cs).borrow().idr.write_with_zero(|w| {
                    w.wakeup()
                        .clear_bit()
                        .rxrsm()
                        .clear_bit()
                        .extrsm()
                        .clear_bit()
                });

                // Ack suspend just in case (we're enabling it)
                self.udp
                    .borrow(cs)
                    .borrow()
                    .icr
                    .write_with_zero(|w| w.rxsusp().set_bit());

                // Enabling suspend and sof interrupts
                self.udp
                    .borrow(cs)
                    .borrow()
                    .ier
                    .write_with_zero(|w| w.rxsusp().set_bit().sofint().set_bit());
            });

            defmt::info!("{} UdpBus::poll() -> Resume", frm_num());
            return PollResult::Resume;
        }

        // Process suspend interrupt
        if imr.rxsusp().bit() && isr.rxsusp().bit() {
            cortex_m::interrupt::free(|cs| {
                // Clear Suspend interrput
                self.udp
                    .borrow(cs)
                    .borrow()
                    .icr
                    .write_with_zero(|w| w.rxsusp().set_bit());

                // Disable Suspend interrupt
                self.udp
                    .borrow(cs)
                    .borrow()
                    .idr
                    .write_with_zero(|w| w.rxsusp().clear_bit());

                // Enable Resume/External Resume/Wake up interrupts
                self.udp
                    .borrow(cs)
                    .borrow()
                    .ier
                    .write_with_zero(|w| w.wakeup().set_bit().rxrsm().set_bit().extrsm().set_bit());
            });

            // Disable UDP MCK (after this, cannot read from UDP registers)
            self.disable_periph_clk();

            defmt::info!("{} UdpBus::poll() -> Suspend", frm_num());
            return PollResult::Suspend;
        }

        // Check for bus reset interrupt
        if isr.endbusres().bit() {
            // Clear End of BUS Reset
            cortex_m::interrupt::free(|cs| {
                self.udp
                    .borrow(cs)
                    .borrow()
                    .icr
                    .write_with_zero(|w| w.endbusres().set_bit());
            });

            defmt::warn!("{} UdpBus::poll() -> Reset", frm_num());
            return PollResult::Reset;
        }

        PollResult::None
    }

    /// Initiates the remote wakeup sequenec
    fn remote_wakeup(&self) -> usb_device::Result<()> {
        // Enable UDP MCK (from MCK)
        self.enable_periph_clk();

        cortex_m::interrupt::free(|cs| {
            // Check if bus has been suspended
            // NOTE: You must wait 5 ms between host suspending the bus and initiating a remote wakeup
            // The transceiver is disabled on bus suspend, so this is a reliable check
            if !self.udp.borrow(cs).borrow().txvc.read().txvdis().bit() {
                return Err(usb_device::UsbError::NotSuspended);
            }

            // Initiate remote wakeup
            self.udp
                .borrow(cs)
                .borrow()
                .glb_stat
                .modify(|_, w| w.esr().set_bit());
            Ok(())
        })
    }

    /// Simulates disconnection from the USB bus
    fn force_reset(&self) -> usb_device::Result<()> {
        defmt::trace!("{} UdpBus::force_reset()", frm_num());
        self.reset();
        self.disable();

        // Need to wait for the USB device to disconnect
        let freq = crate::clock::get_master_clock_frequency();
        cortex_m::asm::delay(freq.0 / 1000); // 1 ms

        self._enable();
        Ok(())
    }
}
