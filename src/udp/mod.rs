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
