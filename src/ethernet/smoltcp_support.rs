use super::{Controller, Receiver, Transmitter};
use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken};
use smoltcp::time::Instant;
use smoltcp::Error;

impl<'d, 'rxtx, RX: 'd + Receiver, TX: 'd + Transmitter, const PHYADDRESS: u8> Device<'d>
    for Controller<'rxtx, RX, TX, PHYADDRESS>
{
    type RxToken = EthRxToken<'d, RX>;
    type TxToken = EthTxToken<'d, TX>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = super::MTU as usize;
        caps.max_burst_size = Some(1);
        caps
    }

    fn receive(&'d mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        Some((EthRxToken(self.rx), EthTxToken(self.tx)))
    }

    fn transmit(&'d mut self) -> Option<Self::TxToken> {
        Some(EthTxToken(self.tx))
    }
}

pub struct EthRxToken<'a, RX: Receiver>(&'a mut RX);

impl<'a, RX: Receiver> RxToken for EthRxToken<'a, RX> {
    fn consume<R, F>(mut self, _timestamp: Instant, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> Result<R, Error>,
    {
        self.0.receive(f)
    }
}

pub struct EthTxToken<'a, TX>(&'a mut TX);

impl<'a, TX: Transmitter> TxToken for EthTxToken<'a, TX> {
    fn consume<R, F>(self, _timestamp: Instant, size: usize, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> Result<R, Error>,
    {
        self.0.send(size, f)
    }
}
