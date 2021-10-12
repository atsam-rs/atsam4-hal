//! PDC Traits for use with PDC-enabled peripherals
//!
//! Common interface to use with PDC enabled peripherals
//!
//! TODO: Currently only implements Rx mode (e.g. Adc)

use core::marker::PhantomData;
use core::sync::atomic::{self, compiler_fence, Ordering};
use core::{mem, ptr};
use embedded_dma::{StaticReadBuffer, StaticWriteBuffer};

/// Read transfer
pub struct R;

/// Write transfer
pub struct W;

/// DMA Receiver
pub struct RxDma<PAYLOAD> {
    pub(crate) payload: PAYLOAD,
}

pub trait Receive {
    type TransmittedWord;
}

pub trait Transmit {
    type ReceivedWord;
}

/// Trait for DMA readings from peripheral to memory.
pub trait ReadDma<B, RS>: Receive
where
    B: StaticWriteBuffer<Word = RS>,
    Self: core::marker::Sized + TransferPayload,
{
    fn read(self, buffer: B) -> Transfer<W, B, Self>;
    fn read_paused(self, buffer: B) -> Transfer<W, B, Self>;
}

/// Trait for DMA writing from memory to peripheral.
pub trait WriteDma<B, TS>: Transmit
where
    B: StaticReadBuffer<Word = TS>,
    Self: core::marker::Sized + TransferPayload,
{
    fn write(self, buffer: B) -> Transfer<R, B, Self>;
}

/// Trait for DMA simultaneously reading and writing within one synchronous operation. Panics if both buffers are not of equal length.
pub trait ReadWriteDma<RXB, TXB, TS>: Transmit
where
    RXB: StaticWriteBuffer<Word = TS>,
    TXB: StaticReadBuffer<Word = TS>,
    Self: core::marker::Sized + TransferPayload,
{
    fn read_write(self, rx_buffer: RXB, tx_buffer: TXB) -> Transfer<W, (RXB, TXB), Self>;
}

pub trait TransferPayload {
    fn start(&mut self);
    fn stop(&mut self);
    fn in_progress(&self) -> bool;
}

pub struct Transfer<MODE, BUFFER, PAYLOAD>
where
    PAYLOAD: TransferPayload,
{
    _mode: PhantomData<MODE>,
    buffer: BUFFER,
    payload: PAYLOAD,
}

/* TODO Needed for pdc_tx
impl<BUFFER, PAYLOAD> Transfer<R, BUFFER, PAYLOAD>
where
    PAYLOAD: TransferPayload,
{
    pub(crate) fn r(buffer: BUFFER, payload: PAYLOAD) -> Self {
        Transfer {
            _mode: PhantomData,
            buffer,
            payload,
        }
    }
}
*/

impl<BUFFER, PAYLOAD> Transfer<W, BUFFER, PAYLOAD>
where
    PAYLOAD: TransferPayload,
{
    pub(crate) fn w(buffer: BUFFER, payload: PAYLOAD) -> Self {
        Transfer {
            _mode: PhantomData,
            buffer,
            payload,
        }
    }
}

impl<MODE, BUFFER, PAYLOAD> Drop for Transfer<MODE, BUFFER, PAYLOAD>
where
    PAYLOAD: TransferPayload,
{
    fn drop(&mut self) {
        self.payload.stop();
        compiler_fence(Ordering::SeqCst);
    }
}

impl<BUFFER, PAYLOAD, MODE> Transfer<MODE, BUFFER, RxDma<PAYLOAD>>
where
    RxDma<PAYLOAD>: TransferPayload,
{
    pub fn is_done(&self) -> bool {
        !self.payload.in_progress()
    }

    pub fn wait(mut self) -> (BUFFER, RxDma<PAYLOAD>) {
        while !self.is_done() {}

        atomic::compiler_fence(Ordering::Acquire);

        self.payload.stop();

        // we need a read here to make the Acquire fence effective
        // we do *not* need this if `dma.stop` does a RMW operation
        unsafe {
            ptr::read_volatile(&0);
        }

        // we need a fence here for the same reason we need one in `Transfer.wait`
        atomic::compiler_fence(Ordering::Acquire);

        // `Transfer` needs to have a `Drop` implementation, because we accept
        // managed buffers that can free their memory on drop. Because of that
        // we can't move out of the `Transfer`'s fields, so we use `ptr::read`
        // and `mem::forget`.
        //
        // NOTE(unsafe) There is no panic branch between getting the resources
        // and forgetting `self`.
        unsafe {
            let buffer = ptr::read(&self.buffer);
            let payload = ptr::read(&self.payload);
            mem::forget(self);
            (buffer, payload)
        }
    }

    pub fn pause(&mut self) {
        self.payload.stop();
    }

    pub fn resume(&mut self) {
        self.payload.start();
    }
}

macro_rules! pdc_rx {
    (
        $Periph:ident: $periph:ident
    ) => {
        impl $Periph {
            /// Sets the PDC receive address pointer
            pub fn set_receive_address(&mut self, address: u32) {
                self.$periph
                    .rpr
                    .write(|w| unsafe { w.rxptr().bits(address) });
            }

            /// Sets the receive increment counter
            /// Will increment by the count * size of the peripheral data
            pub fn set_receive_counter(&mut self, count: u16) {
                self.$periph.rcr.write(|w| unsafe { w.rxctr().bits(count) });
            }

            /// Sets the PDC receive next address pointer
            pub fn set_receive_next_address(&mut self, address: u32) {
                self.$periph
                    .rnpr
                    .write(|w| unsafe { w.rxnptr().bits(address) });
            }

            /// Sets the receive next increment counter
            /// Will increment by the count * size of the peripheral data
            pub fn set_receive_next_counter(&mut self, count: u16) {
                self.$periph
                    .rncr
                    .write(|w| unsafe { w.rxnctr().bits(count) });
            }

            /// Starts the PDC transfer
            pub fn start_rx_pdc(&mut self) {
                unsafe { self.$periph.ptcr.write_with_zero(|w| w.rxten().set_bit()) };
            }

            /// Stops the PDC transfer
            pub fn stop_rx_pdc(&mut self) {
                unsafe { self.$periph.ptcr.write_with_zero(|w| w.rxtdis().set_bit()) };
            }

            /// Returns `true` if the PDC is active and may be receiving data
            pub fn active_rx_pdc(&self) -> bool {
                self.$periph.ptsr.read().rxten().bit()
            }

            /// Returns `true` if DMA is still in progress
            /// Uses rxbuff, which checks both receive and receive next counters to see if they are 0
            pub fn rx_in_progress(&self) -> bool {
                !self.$periph.isr.read().rxbuff().bit()
            }

            /// Enable ENDRX (End of Receive) interrupt
            /// Triggered when RCR reaches 0
            pub fn enable_endrx_interrupt(&mut self) {
                unsafe { self.$periph.ier.write_with_zero(|w| w.endrx().set_bit()) };
            }

            /// Disable ENDRX (End of Receive) interrupt
            pub fn disable_endrx_interrupt(&mut self) {
                unsafe { self.$periph.idr.write_with_zero(|w| w.endrx().set_bit()) };
            }

            /// Enable RXBUFF (Receive Buffer Full) interrupt
            /// Triggered when RCR and RNCR reach 0
            pub fn enable_rxbuff_interrupt(&mut self) {
                unsafe { self.$periph.ier.write_with_zero(|w| w.rxbuff().set_bit()) };
            }

            /// Disable RXBUFF (Receive Buffer Full) interrupt
            pub fn disable_rxbuff_interrupt(&mut self) {
                unsafe { self.$periph.idr.write_with_zero(|w| w.rxbuff().set_bit()) };
            }
        }
    };
}
pub(crate) use pdc_rx;

/* TODO Commenting out until first usage (likely SPI)
macro_rules! pdc_tx {
    (
        $Periph:ident: $periph:ident
    ) => {
        impl $Periph {
            /// Sets the PDC transmit address pointer
            pub fn set_transmit_address(&mut self, address: u32) {
                self.$periph
                    .rpr
                    .write(|w| unsafe { w.txptr().bits(address) });
            }

            /// Sets the transmit increment counter
            /// Will increment by the count * size of the peripheral data
            pub fn set_transmit_counter(&mut self, count: u16) {
                self.$periph.rcr.write(|w| unsafe { w.txctr().bits(count) });
            }

            /// Sets the PDC transmit next address pointer
            pub fn set_transmit_next_address(&mut self, address: u32) {
                self.$periph
                    .rnpr
                    .write(|w| unsafe { w.txnptr().bits(address) });
            }

            /// Sets the transmit next increment counter
            /// Will increment by the count * size of the peripheral data
            pub fn set_transmit_next_counter(&mut self, count: u16) {
                self.$periph
                    .rncr
                    .write(|w| unsafe { w.txnctr().bits(count) });
            }

            /// Starts the PDC transfer
            pub fn start_tx_pdc(&mut self) {
                self.$periph.ptcr.write_with_zero(|w| w.txten().set_bit());
            }

            /// Stops the PDC transfer
            pub fn stop_tx_pdc(&mut self) {
                self.$periph.ptcr.write_with_zero(|w| w.txtdis().set_bit());
            }

            /// Returns `true` if the PDC is active and may be receiving data
            pub fn active_tx_pdc(&self) -> bool {
                self.$periph.ptsr.read().txten().bit()
            }

            /// Returns `true` if DMA is still in progress
            /// Uses rxbuff, which checks both transmit and transmit next counters to see if they are 0
            pub fn tx_in_progress(&self) -> bool {
                !self.$periph.isr.read().txbufe().bit()
            }

            /// Enable ENDRX (End of Transmit) interrupt
            /// Triggered when RCR reaches 0
            pub fn enable_endrx_interrupt(&mut self) {
                self.$periph.ier.write_with_zero(|w| w.endtx().set_bit());
            }

            /// Disable ENDRX (End of Transmit) interrupt
            pub fn disable_endrx_interrupt(&mut self) {
                self.$periph.idr.write_with_zero(|w| w.endtx().set_bit());
            }

            /// Enable RXBUFF (Transmit Buffer Full) interrupt
            /// Triggered when RCR and RNCR reach 0
            pub fn enable_rxbuff_interrupt(&mut self) {
                self.$periph.ier.write_with_zero(|w| w.txbufe().set_bit());
            }

            /// Disable RXBUFF (Transmit Buffer Full) interrupt
            pub fn disable_rxbuff_interrupt(&mut self) {
                self.$periph.idr.write_with_zero(|w| w.txbufe().set_bit());
            }
        }
    };
}
pub(crate) use pdc_tx;
*/
