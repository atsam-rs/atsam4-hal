use super::{Controller, Receiver, Transmitter, MTU};
use crate::pac::GMAC;
use smoltcp::phy::{Device, DeviceCapabilities, RxToken, TxToken};
use smoltcp::time::Instant;
use smoltcp::Error;

impl<'a, 'rxtx: 'a> Device<'a> for Controller<'rxtx> {
    type RxToken = EthRxToken<'a, 'rxtx>;
    type TxToken = EthTxToken<'a, 'rxtx>;

    fn capabilities(&self) -> DeviceCapabilities {
        let mut caps = DeviceCapabilities::default();
        caps.max_transmission_unit = MTU as usize;
        caps.max_burst_size = Some(1);
        caps
    }

    fn receive(&'a mut self) -> Option<(Self::RxToken, Self::TxToken)> {
        match self.rx.can_receive() {
            true => Some((EthRxToken(&self.rx), EthTxToken(&self.tx, &self.gmac))),
            false => None,
        }
    }

    fn transmit(&'a mut self) -> Option<Self::TxToken> {
        Some(EthTxToken(&self.tx, &self.gmac))
    }
}

trait SmolTcpReceiver {
    fn receive_smoltcp<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &self,
        f: F,
    ) -> Result<R, smoltcp::Error>;
}

impl<'rx> SmolTcpReceiver for Receiver<'rx> {
    fn receive_smoltcp<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &self,
        f: F,
    ) -> Result<R, smoltcp::Error> {
        let mut buffer: [u8; MTU] = [0; MTU];
        match self.receive(&mut buffer[..]) {
            Ok(size) => f(&mut buffer[..size]),
            Err(err) => match err {
                nb::Error::WouldBlock => Err(smoltcp::Error::Exhausted),
                e => panic!("{:?}", e),
            },
        }
    }
}

trait SmolTcpTransmitter {
    fn send_smoltcp<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &self,
        gmac: &GMAC,
        size: usize,
        f: F,
    ) -> Result<R, smoltcp::Error>;
}

impl<'tx> SmolTcpTransmitter for Transmitter<'tx> {
    fn send_smoltcp<R, F: FnOnce(&mut [u8]) -> Result<R, smoltcp::Error>>(
        &self,
        gmac: &GMAC,
        size: usize,
        f: F,
    ) -> Result<R, smoltcp::Error> {
        let mut buffer: [u8; MTU] = [0; MTU];
        let r = f(&mut buffer[..size])?;

        match self.send(&gmac, &buffer[..size]) {
            Ok(_) => Ok(r),
            Err(err) => match err {
                nb::Error::WouldBlock => Err(smoltcp::Error::Exhausted),
                e => panic!("{:?}", e),
            },
        }
    }
}

pub struct EthRxToken<'a, 'rxtx>(&'a Receiver<'rxtx>);
impl<'a, 'rxtx> RxToken for EthRxToken<'a, 'rxtx> {
    fn consume<R, F>(self, _timestamp: Instant, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> Result<R, Error>,
    {
        self.0.receive_smoltcp(f)
    }
}

pub struct EthTxToken<'a, 'rxtx>(&'a Transmitter<'rxtx>, &'a GMAC);
impl<'a, 'rxtx> TxToken for EthTxToken<'a, 'rxtx> {
    fn consume<R, F>(self, _timestamp: Instant, size: usize, f: F) -> Result<R, Error>
    where
        F: FnOnce(&mut [u8]) -> Result<R, Error>,
    {
        self.0.send_smoltcp(self.1, size, f)
    }
}
