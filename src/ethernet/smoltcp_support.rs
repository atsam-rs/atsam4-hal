use crate::pac::GMAC;
use super::{Controller, Receiver, Transmitter};
use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken};
use smoltcp::time::Instant;
use smoltcp::Error;

impl<'d, 'rxtx: 'd> Device<'d> for Controller<'d> {
    type RxToken = EthRxToken<'d>;
    type TxToken = EthTxToken<'d>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = super::MTU as usize;
        caps.max_burst_size = Some(1);
        caps
    }

    fn receive(&'d mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        match self.rx.can_receive() {
            true => Some((EthRxToken(&mut self.rx), EthTxToken(&mut self.tx, &self.gmac))),
            false => None,
        }
    }

    fn transmit(&'d mut self) -> Option<Self::TxToken> {
        Some(EthTxToken(&mut self.tx, &self.gmac))
    }
}

pub struct EthRxToken<'a>(&'a mut Receiver<'a>);
impl<'a> RxToken for EthRxToken<'a> {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> Result<R, Error>,
    {
        self.0.receive_smoltcp(f)
    }
}

pub struct EthTxToken<'a>( &'a mut Transmitter<'a>, &'a GMAC);
impl<'a> TxToken for EthTxToken<'a> {
    fn consume<R, F>(self, _timestamp: Instant, size: usize, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> Result<R, Error>,
    {
        self.0.send_smoltcp(self.1, size, f)
    }
}
