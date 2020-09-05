#![no_std]
#![no_main]

extern crate atsam4_hal as hal;

#[cfg(not(feature = "use_semihosting"))]
extern crate panic_halt;
#[cfg(feature = "use_semihosting")]
extern crate panic_semihosting;

use hal::clock::ClockController;
use hal::pac::{CorePeripherals, Peripherals};
use hal::prelude::*;

#[entry]
fn main() -> ! {
    let mut peripherals = Peripherals::take().unwrap();
    let core = CorePeripherals::take().unwrap();

    let mut clocks = ClockController::with_internal_32kosc(
        peripherals.GCLK,
        &mut peripherals.PM,
        &mut peripherals.SYSCTRL,
        &mut peripherals.NVMCTRL,
    );

    // let mut pins = hal::Pins::new(peripherals.PORT);
    // let mut red_led = pins.d2.into_open_drain_output(&mut pins.port);
    // let mut delay = Delay::new(core.SYST, &mut clocks);

    // loop {
    //     delay.delay_ms(200u8);
    //     red_led.set_high().unwrap();
    //     delay.delay_ms(200u8);
    //     red_led.set_low().unwrap();
    // }
}
