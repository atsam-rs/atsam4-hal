//! UDP (USB Device Port) Implementation
//! NOTE: From ASF the following MCUs could possibly be supported with this module
//!       atsam3s
//!       atsam4e (supported)
//!       atsam4s (supported)
//!       atsamg55

pub use usb_device;

mod bus;
pub use self::bus::UdpBus;

mod endpoint;
pub use self::endpoint::Endpoint;

use crate::pac::UDP;
use crate::BorrowUnchecked;

/// Retrieve current frame number (updated on SOF_EOP)
pub fn frm_num() -> u16 {
    UDP::borrow_unchecked(|udp| udp.frm_num.read().frm_num().bits())
}
