use super::{Controller, Receiver, Transmitter};
use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken};
use smoltcp::time::Instant;
use smoltcp::Error;

impl<'d, 'rxtx, RX: Receiver, TX: Transmitter> Device<'d> for Controller<'rxtx, RX, TX> {
    type RxToken = EthRxToken;
    type TxToken = EthTxToken;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = super::MTU as usize;
        caps.max_burst_size = Some(1);
        caps
    }

    fn receive(&mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        None
    }

    fn transmit(&mut self) -> Option<Self::TxToken> {
        None
    }
}

pub struct EthRxToken {
}

impl RxToken for EthRxToken {
    fn consume<R, F>(mut self, _timestamp: Instant, _f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> Result<R, Error>,
    {
        unimplemented!();
    }
}

pub struct EthTxToken {
}

impl TxToken for EthTxToken {
    fn consume<R, F>(self, _timestamp: Instant, _len: usize, _f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> Result<R, Error>,
    {
        unimplemented!();
    }
}
