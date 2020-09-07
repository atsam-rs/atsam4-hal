#![no_std]
#![no_main]

extern crate cortex_m;
extern crate cortex_m_rt;
extern crate cortex_m_semihosting;
extern crate atsam4_hal as hal;
extern crate embedded_hal;

#[cfg(not(feature = "use_semihosting"))]
extern crate panic_halt;
#[cfg(feature = "use_semihosting")]
extern crate panic_semihosting;

use cortex_m_rt::entry;
use hal::clock::ClockController;
use hal::pac::{CorePeripherals, Peripherals};
use hal::delay::Delay;
use embedded_hal::prelude::*;
use cortex_m_semihosting::hprintln;

#[entry]
fn main() -> ! {
    let core = CorePeripherals::take().unwrap();

    let mut peripherals = Peripherals::take().unwrap();
    let clocks = ClockController::with_internal_32kosc(
        peripherals.PMC,
        &mut peripherals.EFC
    );

    let mut delay = Delay::new(core.SYST, clocks.into());

    loop {
        hprintln!("This message will repeat every second.").ok();
        delay.delay_ms(1000u32);
    }
}
