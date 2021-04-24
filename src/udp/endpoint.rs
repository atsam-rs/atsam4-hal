use crate::pac::generic::Variant::Val;
use crate::pac::udp::csr::EPTYPE_A;
use crate::pac::UDP;
use crate::BorrowUnchecked;
use usb_device::{
    bus::PollResult,
    endpoint::{EndpointAddress, EndpointType},
    UsbDirection,
};

/// Needed to support ping-pong buffers
/// If interrupts are processed too slowly, it's possible that both Rx banks have been filled.
/// There is no way from the register interface to know which buffer is first so we have to
/// keep track of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NextBank {
    Bank0,
    Bank1,
}

/// atsam4 has two registers dedicated to each endpoint
/// CSRx and FDRx
/// Most of the relevant information about the endpoint can be queried directly from the registers.
/// TODO: Isochronous not implemented
pub struct Endpoint {
    index: u8,
    interval: u8,
    max_packet_size: u16,
    ep_type: EndpointType,
    ep_dir: UsbDirection,
    next_bank: NextBank,
    allocated: bool,
    txbanks_free: u8,
}

macro_rules! clear_ep {
    (
        $udp:ident,
        $ep:ident
    ) => {{
        // Set
        $udp.rst_ep.modify(|_, w| w.$ep().set_bit());
        // Wait for clear to finish
        while !$udp.rst_ep.read().$ep().bit() {}
        // Clear
        $udp.rst_ep.modify(|_, w| w.$ep().clear_bit());
    }};
}

impl Endpoint {
    pub fn new(index: u8) -> Self {
        // TODO Figure out how to given ownership to specific CSR and FDR registers
        //      These are effectively owned by this struct, but I'm not sure how to do
        //      this with how svd2rust generated
        log::trace!("Endpoint::new({})", index);

        // Disable endpoint (to start fresh)
        UDP::borrow_unchecked(|udp| {
            udp.csr_mut()[index as usize].modify(|_, w| w.epeds().clear_bit())
        });

        Self {
            index,
            interval: 1,
            max_packet_size: 8,
            ep_type: EndpointType::Interrupt,
            ep_dir: UsbDirection::Out,
            next_bank: NextBank::Bank0,
            allocated: false,
            txbanks_free: 0,
        }
    }

    /// Allocates the endpoint
    /// Since atsam4 uses registers for the buffer and configuration no memory
    /// is allocated. However there is a finite number of endpoints so we still
    /// need to do allocation and configuration.
    pub fn alloc(
        &mut self,
        ep_type: EndpointType,
        ep_dir: UsbDirection,
        max_packet_size: u16,
        interval: u8,
    ) -> usb_device::Result<EndpointAddress> {
        log::trace!(
            "Endpoint{}::alloc({:?}, {:?}, {}, {})",
            self.index,
            ep_type,
            ep_dir,
            max_packet_size,
            interval
        );

        let address = EndpointAddress::from_parts(self.index as usize, ep_dir);
        // Ignore allocation for Control IN endpoints (but only if this endpoint can be a
        // control endpoint).
        if ep_type == EndpointType::Control && ep_dir == UsbDirection::In && !self.dual_bank() {
            log::trace!("Endpoint{}::alloc() -> {:?}", self.index, address,);
            return Ok(address);
        }

        // Already allocated
        if self.allocated {
            return Err(usb_device::UsbError::InvalidEndpoint);
        }

        // Check if max_packet_size will fit on this endpoint
        self.max_packet_size = max_packet_size;
        if max_packet_size > self.max_packet_size() {
            return Err(usb_device::UsbError::EndpointMemoryOverflow);
        }

        // Check if endpoint type can be allocated and set register
        match ep_type {
            EndpointType::Bulk => {}
            EndpointType::Control => {
                // Control endpoints are only valid on ep0 and ep3 (non-dual bank)
                if self.dual_bank() {
                    return Err(usb_device::UsbError::Unsupported);
                }
            }
            EndpointType::Interrupt => {}
            EndpointType::Isochronous => {
                // Must have dual banks for isochronous support
                if !self.dual_bank() {
                    return Err(usb_device::UsbError::Unsupported);
                }
            }
        }
        self.ep_type = ep_type;
        self.ep_dir = ep_dir;

        // Set free tx banks
        self.txbanks_free = if self.dual_bank() { 2 } else { 1 };

        self.allocated = true;
        self.interval = interval;
        log::trace!("Endpoint{}::alloc() -> {:?}", self.index, address,);
        Ok(address)
    }

    /// Gets the endpoint address including direction bit.
    pub fn address(&self) -> EndpointAddress {
        EndpointAddress::from_parts(self.index as usize, self.ep_dir)
    }

    /// Gets the maximum packet size for the endpoint.
    pub fn max_packet_size(&self) -> u16 {
        let hardware = match self.index {
            0..=3 | 6 | 7 => 64,
            4 | 5 => 512, // Only really useful for isochronous
            _ => 0,       // Invalid
        };
        core::cmp::min(hardware, self.max_packet_size)
    }

    /// Check if endpoint is dual-buffered
    fn dual_bank(&self) -> bool {
        !matches!(self.index, 0 | 3)
    }

    /// Check endpoint interrupt
    fn interrupt(&self) -> bool {
        let isr = UDP::borrow_unchecked(|udp| udp.isr.read());
        match self.index {
            0 => isr.ep0int().bit(),
            1 => isr.ep1int().bit(),
            2 => isr.ep2int().bit(),
            3 => isr.ep3int().bit(),
            4 => isr.ep4int().bit(),
            5 => isr.ep5int().bit(),
            6 => isr.ep6int().bit(),
            7 => isr.ep7int().bit(),
            _ => false, // Invalid
        }
    }

    /// Check if endpoint is enabled
    fn interrupt_enabled(&self) -> bool {
        let imr = UDP::borrow_unchecked(|udp| udp.imr.read());
        match self.index {
            0 => imr.ep0int().bit(),
            1 => imr.ep1int().bit(),
            2 => imr.ep2int().bit(),
            3 => imr.ep3int().bit(),
            4 => imr.ep4int().bit(),
            5 => imr.ep5int().bit(),
            6 => imr.ep6int().bit(),
            7 => imr.ep7int().bit(),
            _ => false, // Invalid
        }
    }

    /// Set interrupt (enable/disable)
    fn interrupt_set(&self, enable: bool) {
        // Enable interrupt for endpoint
        UDP::borrow_unchecked(|udp| {
            if enable {
                match self.index {
                    0 => udp.ier.write_with_zero(|w| w.ep0int().set_bit()),
                    1 => udp.ier.write_with_zero(|w| w.ep1int().set_bit()),
                    2 => udp.ier.write_with_zero(|w| w.ep2int().set_bit()),
                    3 => udp.ier.write_with_zero(|w| w.ep3int().set_bit()),
                    4 => udp.ier.write_with_zero(|w| w.ep4int().set_bit()),
                    5 => udp.ier.write_with_zero(|w| w.ep5int().set_bit()),
                    6 => udp.ier.write_with_zero(|w| w.ep6int().set_bit()),
                    7 => udp.ier.write_with_zero(|w| w.ep7int().set_bit()),
                    _ => {} // Invalid
                }
            } else {
                match self.index {
                    0 => udp.idr.write_with_zero(|w| w.ep0int().set_bit()),
                    1 => udp.idr.write_with_zero(|w| w.ep1int().set_bit()),
                    2 => udp.idr.write_with_zero(|w| w.ep2int().set_bit()),
                    3 => udp.idr.write_with_zero(|w| w.ep3int().set_bit()),
                    4 => udp.idr.write_with_zero(|w| w.ep4int().set_bit()),
                    5 => udp.idr.write_with_zero(|w| w.ep5int().set_bit()),
                    6 => udp.idr.write_with_zero(|w| w.ep6int().set_bit()),
                    7 => udp.idr.write_with_zero(|w| w.ep7int().set_bit()),
                    _ => {} // Invalid
                }
            }
        });
    }

    /// Gets the poll interval for interrupt endpoints.
    pub fn interval(&self) -> u8 {
        self.interval
    }

    /// Sets the STALL condition for the endpoint.
    pub fn stall(&self) {
        log::trace!("Endpoint{}::stall()", self.index);
        UDP::borrow_unchecked(|udp| {
            udp.csr_mut()[self.index as usize].modify(|_, w| w.forcestall().set_bit())
        });
    }

    /// Clears the STALL condition of the endpoint.
    pub fn unstall(&self) {
        log::trace!("Endpoint{}::unstall()", self.index);
        UDP::borrow_unchecked(|udp| {
            udp.csr_mut()[self.index as usize].modify(|_, w| w.forcestall().clear_bit())
        });
    }

    /// Check if STALL has been set
    pub fn is_stalled(&self) -> bool {
        UDP::borrow_unchecked(|udp| udp.csr()[self.index as usize].read().forcestall().bit())
    }

    /// Enable endpoint
    pub fn enable(&self) {
        // Only enable if the endpoint has been allocated
        if self.allocated {
            log::trace!("Endpoint{}::enable()", self.index);
            UDP::borrow_unchecked(|udp| {
                udp.csr_mut()[self.index as usize].modify(|_, w| w.epeds().set_bit())
            });
        }
    }

    /// Disable endpoint
    pub fn disable(&self) {
        log::trace!("Endpoint{}::disable()", self.index);
        UDP::borrow_unchecked(|udp| {
            udp.csr_mut()[self.index as usize].modify(|_, w| w.epeds().clear_bit())
        });
    }

    /// Clear fifo (two step, set then clear)
    fn clear_fifo(&self) {
        UDP::borrow_unchecked(|udp| {
            match self.index {
                0 => clear_ep!(udp, ep0),
                1 => clear_ep!(udp, ep1),
                2 => clear_ep!(udp, ep2),
                3 => clear_ep!(udp, ep3),
                4 => clear_ep!(udp, ep4),
                5 => clear_ep!(udp, ep5),
                6 => clear_ep!(udp, ep6),
                7 => clear_ep!(udp, ep7),
                _ => {} // Invalid
            }
        });
    }

    /// Reset endpoint to allocated settings
    pub fn reset(&mut self) {
        if !self.allocated {
            return;
        }
        log::trace!("Endpoint{}::reset()", self.index);

        // Reset endpoint FIFO
        self.clear_fifo();

        // Clear free tx banks
        self.txbanks_free = if self.dual_bank() { 2 } else { 1 };

        // Setup CSR
        match self.ep_type {
            EndpointType::Bulk => match self.ep_dir {
                UsbDirection::In => {
                    UDP::borrow_unchecked(|udp| {
                        udp.csr()[self.index as usize].modify(|_, w| w.eptype().bulk_in())
                    });
                }
                UsbDirection::Out => {
                    UDP::borrow_unchecked(|udp| {
                        udp.csr()[self.index as usize].modify(|_, w| w.eptype().bulk_out())
                    });
                }
            },
            EndpointType::Control => {
                // Control Endpoint must start out configured in the OUT direction
                UDP::borrow_unchecked(|udp| {
                    udp.csr()[self.index as usize]
                        .modify(|_, w| w.eptype().ctrl().dir().clear_bit())
                });
            }
            EndpointType::Interrupt => match self.ep_dir {
                UsbDirection::In => {
                    UDP::borrow_unchecked(|udp| {
                        udp.csr()[self.index as usize].modify(|_, w| w.eptype().int_in())
                    });
                }
                UsbDirection::Out => {
                    UDP::borrow_unchecked(|udp| {
                        udp.csr()[self.index as usize].modify(|_, w| w.eptype().int_out())
                    });
                }
            },
            EndpointType::Isochronous => match self.ep_dir {
                UsbDirection::In => {
                    UDP::borrow_unchecked(|udp| {
                        udp.csr()[self.index as usize].modify(|_, w| w.eptype().iso_in())
                    });
                }
                UsbDirection::Out => {
                    UDP::borrow_unchecked(|udp| {
                        udp.csr()[self.index as usize].modify(|_, w| w.eptype().iso_out())
                    });
                }
            },
        }

        // Enable endpoint
        self.enable();

        // Enable interrupt
        self.interrupt_set(true);
    }

    /// Poll endpoint
    pub fn poll(&mut self) -> PollResult {
        if !self.allocated {
            return PollResult::None;
        }

        // Check endpoint interrupt
        if self.interrupt_enabled() && self.interrupt() {
            let csr = UDP::borrow_unchecked(|udp| udp.csr()[self.index as usize].read());

            // Determine endpoint type
            match self.ep_type {
                EndpointType::Control => {
                    // SETUP packet received
                    let ep_setup = if csr.rxsetup().bit() {
                        log::trace!("Endpoint{}::Poll(Ctrl) -> SETUP", self.index);
                        1 << self.index
                    } else {
                        0
                    };
                    // IN packet sent
                    let ep_in_complete = if csr.txcomp().bit() {
                        // Ack TXCOMP
                        UDP::borrow_unchecked(|udp| {
                            udp.csr_mut()[self.index as usize].modify(|_, w| w.txcomp().clear_bit())
                        });
                        self.txbanks_free += 1;
                        log::trace!("Endpoint{}::Poll(Ctrl) -> IN", self.index);
                        1 << self.index
                    } else {
                        0
                    };
                    // OUT packet received
                    let ep_out = if csr.rx_data_bk0().bit() {
                        log::trace!("Endpoint{}::Poll(Ctrl) -> OUT", self.index);
                        1 << self.index
                    } else {
                        0
                    };

                    // Return if we found any CTRL status flags
                    if (ep_setup | ep_in_complete | ep_out) > 0 {
                        return PollResult::Data {
                            ep_out,
                            ep_in_complete,
                            ep_setup,
                        };
                    }
                }
                EndpointType::Bulk | EndpointType::Interrupt | EndpointType::Isochronous => {
                    // RXOUT: Full packet received
                    let ep_out = if csr.rx_data_bk0().bit() || csr.rx_data_bk1().bit() {
                        log::trace!("Endpoint{}::Poll({:?}) -> OUT", self.index, self.ep_type);
                        1 << self.index
                    } else {
                        0
                    };
                    // TXIN: Packet sent
                    let ep_in_complete = if csr.txcomp().bit() {
                        // Ack TXCOMP
                        UDP::borrow_unchecked(|udp| {
                            udp.csr_mut()[self.index as usize].modify(|_, w| w.txcomp().clear_bit())
                        });
                        self.txbanks_free += 1;
                        log::trace!("Endpoint{}::Poll({:?}) -> IN", self.index, self.ep_type);
                        1 << self.index
                    } else {
                        0
                    };

                    // Return if we found any data status flags
                    if (ep_in_complete | ep_out) > 0 {
                        return PollResult::Data {
                            ep_out,
                            ep_in_complete,
                            ep_setup: 0,
                        };
                    }
                }
            }

            // STALLed
            if csr.stallsent().bit() {
                // Ack STALL
                UDP::borrow_unchecked(|udp| {
                    udp.csr_mut()[self.index as usize].modify(|_, w| w.stallsent().clear_bit())
                });
            }
        }

        PollResult::None
    }

    /// Writes a single packet of data to the specified endpoint and returns number of bytes
    /// actually written. The buffer must not be longer than the `max_packet_size` specified when
    /// allocating the endpoint.
    ///
    /// # Errors
    ///
    /// Note: USB bus implementation errors are directly passed through, so be prepared to handle
    /// other errors as well.
    ///
    /// * [`WouldBlock`](crate::UsbError::WouldBlock) - The transmission buffer of the USB
    ///   peripheral is full and the packet cannot be sent now. A peripheral may or may not support
    ///   concurrent transmission of packets.
    /// * [`BufferOverflow`](crate::UsbError::BufferOverflow) - The data is longer than the
    ///   `max_packet_size` specified when allocating the endpoint. This is generally an error in
    ///   the class implementation.
    pub fn write(&mut self, data: &[u8]) -> usb_device::Result<usize> {
        log::trace!("Endpoint{}::write({:?})", self.index, data);
        // -- Data IN Transaction --
        // * Check for FIFO ready by polling TXPKTRDY in CSR
        // * Write packet data to FDR
        // * Notify FIFO ready to send by setting TXPKTRDY
        // * FIFO has been released when TXCOMP is set (clear TXCOMP)
        // * Write next packet to FDR
        // * Notify FIFO ready to send by setting TXPKTRDY
        // * After the last packet is sent, clear TXCOMP
        // -- Data IN Transaction (/w Ping-pong) --
        // Isochronous must use Ping-pong for Data IN
        // * Check for FIFO ready by polling TXPKTRDY in CSR
        // * Write packet data to FDR (Bank 0)
        // * Notify FIFO ready to send by setting TXPKTRDY
        // * Immediately write next packet to FDR (Bank 1)
        // * Bank 0 FIFO has been released when TXCOMP is set (clear TXCOMP)
        // * Write next packet to FDR (Bank 0)
        // * Notify FIFO ready to send by setting TXPKTRDY
        // * After the last packet is sent, clear TXCOMP

        // Make sure endpoint has been allocated
        if !self.allocated {
            return Err(usb_device::UsbError::InvalidEndpoint);
        }

        // Make sure FIFO is ready
        // This counter takes into account Ctrl vs Non-Ctrl endpoints
        if self.txbanks_free == 0 {
            return Err(usb_device::UsbError::WouldBlock);
        }

        // Make sure the DIR bit is set correctly for CTRL endpoints
        if self.ep_type == EndpointType::Control {
            UDP::borrow_unchecked(|udp| {
                udp.csr()[self.index as usize].modify(|_, w| w.eptype().ctrl().dir().set_bit())
            });
        }

        // Make sure we don't overflow the endpoint fifo
        // Each EP has a different size and is not configurable
        if data.len() > self.max_packet_size() as usize {
            return Err(usb_device::UsbError::EndpointMemoryOverflow);
        }

        // Write data to fifo
        for byte in data {
            UDP::borrow_unchecked(|udp| {
                udp.fdr[self.index as usize]
                    .write_with_zero(|w| unsafe { w.fifo_data().bits(*byte) })
            });
        }
        self.txbanks_free -= 1;

        // Set TXPKTRDY
        UDP::borrow_unchecked(|udp| {
            udp.csr_mut()[self.index as usize].modify(|_, w| w.txpktrdy().set_bit())
        });

        log::trace!("Endpoint{}::write() -> {}", self.index, data.len());
        Ok(data.len())
    }

    /// Reads a single packet of data from the specified endpoint and returns the actual length of
    /// the packet. The buffer should be large enough to fit at least as many bytes as the
    /// `max_packet_size` specified when allocating the endpoint.
    ///
    /// # Errors
    ///
    /// Note: USB bus implementation errors are directly passed through, so be prepared to handle
    /// other errors as well.
    ///
    /// * [`WouldBlock`](crate::UsbError::WouldBlock) - There is no packet to be read. Note that
    ///   this is different from a received zero-length packet, which is valid and significant in
    ///   USB. A zero-length packet will return `Ok(0)`.
    /// * [`BufferOverflow`](crate::UsbError::BufferOverflow) - The received packet is too long to
    ///   fit in `data`. This is generally an error in the class implementation.
    pub fn read(&mut self, data: &mut [u8]) -> usb_device::Result<usize> {
        log::trace!("Endpoint{}::read()", self.index);
        let csr = UDP::borrow_unchecked(|udp| udp.csr()[self.index as usize].read());

        // Make sure endpoint has been allocated
        if !self.allocated {
            return Err(usb_device::UsbError::InvalidEndpoint);
        }

        // Determine if we've been configured as a control endpoint
        if csr.eptype().variant() == Val(EPTYPE_A::CTRL) {
            // -- Setup Transaction --
            // * Hardware automatically acknowledges
            // * RXSETUP is set in CSR
            // * Interrupt until RXSETUP is cleared
            // -- Data OUT Transaction --
            // * Until FIFO is ready, hardware sends NAKs automatically
            // * After data is written to FIFO, ACK automatically sent
            // * RX_DATA_BK0 is set in CSR
            // * Interrupt until RX_DATA_BK0 is cleared
            // * RXBYTECNT has the number of bytes in the FIFO
            // * Read FDR for FIFO data
            // * Clear RX_DATA_BK0 to indicate finished

            // Check for RXSETUP
            if csr.rxsetup().bit() {
                // Check incoming data length, make sure our buffer is big enough
                let rxbytes = csr.rxbytecnt().bits() as usize;
                if rxbytes > data.len() {
                    // Clear RXSETUP, to continue after the overflow
                    UDP::borrow_unchecked(|udp| {
                        udp.csr_mut()[self.index as usize].modify(|_, w| w.rxsetup().clear_bit())
                    });
                    return Err(usb_device::UsbError::BufferOverflow);
                }

                // Copy fifo into buffer
                for byte in data.iter_mut().take(rxbytes) {
                    *byte = UDP::borrow_unchecked(|udp| {
                        udp.fdr[self.index as usize].read().fifo_data().bits()
                    });
                }

                // Clear RXSETUP
                UDP::borrow_unchecked(|udp| {
                    udp.csr_mut()[self.index as usize].modify(|_, w| w.rxsetup().clear_bit())
                });
                log::trace!("Endpoint{}::read({:?}) SETUP", self.index, data);
                return Ok(rxbytes);
            }

            // Check for OUT packet in Bank0
            if csr.rx_data_bk0().bit() {
                // Check incoming data length, make sure our buffer is big enough
                let rxbytes = csr.rxbytecnt().bits() as usize;
                if rxbytes > data.len() {
                    // Clear RX_DATA_BK0, to continue after the overflow
                    UDP::borrow_unchecked(|udp| {
                        udp.csr_mut()[self.index as usize]
                            .modify(|_, w| w.rx_data_bk0().clear_bit())
                    });
                    return Err(usb_device::UsbError::BufferOverflow);
                }

                // Copy fifo into buffer
                for byte in data.iter_mut().take(rxbytes) {
                    *byte = UDP::borrow_unchecked(|udp| {
                        udp.fdr[self.index as usize].read().fifo_data().bits()
                    });
                }

                // Clear RX_DATA_BK0
                UDP::borrow_unchecked(|udp| {
                    udp.csr_mut()[self.index as usize].modify(|_, w| w.rx_data_bk0().clear_bit())
                });
                log::trace!("Endpoint{}::read({:?}) OUT", self.index, data);
                return Ok(rxbytes);
            }

            // No data
            return Err(usb_device::UsbError::WouldBlock);
        }

        // Make sure this is an Out endpoint
        match csr.eptype().variant() {
            Val(EPTYPE_A::BULK_OUT) | Val(EPTYPE_A::INT_OUT) | Val(EPTYPE_A::ISO_OUT) => {}
            _ => {
                return Err(usb_device::UsbError::InvalidEndpoint);
            }
        }

        // -- Data OUT Transaction (/w Ping-pong) --
        // Isochronous must use Ping-pong for Data OUT
        // NOTE: Must keep track of which bank should be next as there's no way
        //       to determine which bank should be next if the interrupt was slow.
        // * Until FIFO is ready, hardware sends NAKs automatically
        // * After data is written to FIFO, ACK automatically sent
        //   - Host can immediately start sending data to Bank 1 after ACK
        // * RX_DATA_BK0 is set in CSR
        // * Interrupt until RX_DATA_BK0 is cleared
        // * RXBYTECNT has the number of bytes in the FIFO
        // * Read FDR for FIFO data
        // * Clear RX_DATA_BK0 to indicate finished
        //   - Host can begin sending data to Bank 0

        // Determine which bank to read
        let bank = if csr.rx_data_bk0().bit() && csr.rx_data_bk1().bit() {
            // Both banks are ready, use prior state to decide
            self.next_bank
        } else if csr.rx_data_bk0().bit() {
            NextBank::Bank0
        // EP0 and EP3 are not dual-buffered
        } else if !self.dual_bank() && csr.rx_data_bk1().bit() {
            NextBank::Bank1
        } else {
            // No data
            return Err(usb_device::UsbError::WouldBlock);
        };

        // Check incoming data length, make sure our buffer is big enough
        let rxbytes = csr.rxbytecnt().bits() as usize;
        if rxbytes > data.len() {
            // Clear bank fifo, to continue after the overflow
            match bank {
                NextBank::Bank0 => {
                    UDP::borrow_unchecked(|udp| {
                        udp.csr_mut()[self.index as usize]
                            .modify(|_, w| w.rx_data_bk0().clear_bit())
                    });
                    self.next_bank = NextBank::Bank1;
                }
                NextBank::Bank1 => {
                    UDP::borrow_unchecked(|udp| {
                        udp.csr_mut()[self.index as usize]
                            .modify(|_, w| w.rx_data_bk1().clear_bit())
                    });
                    self.next_bank = NextBank::Bank0;
                }
            }
            return Err(usb_device::UsbError::BufferOverflow);
        }

        // Copy fifo into buffer
        for byte in data.iter_mut().take(rxbytes) {
            *byte =
                UDP::borrow_unchecked(|udp| udp.fdr[self.index as usize].read().fifo_data().bits());
        }

        // Clear bank fifo to indicate finished
        match bank {
            NextBank::Bank0 => {
                UDP::borrow_unchecked(|udp| {
                    udp.csr_mut()[self.index as usize].modify(|_, w| w.rx_data_bk0().clear_bit())
                });
                self.next_bank = NextBank::Bank1;
            }
            NextBank::Bank1 => {
                UDP::borrow_unchecked(|udp| {
                    udp.csr_mut()[self.index as usize].modify(|_, w| w.rx_data_bk1().clear_bit())
                });
                self.next_bank = NextBank::Bank0;
            }
        }
        Ok(rxbytes)
    }
}
