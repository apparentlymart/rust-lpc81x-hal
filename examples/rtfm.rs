//! Illustrates using lpc81x-hal with "Realtime for the Masses" (RTFM).

#![no_std]
#![no_main]

extern crate cortex_m_rt;
//extern crate panic_halt;

use lpc81x_hal as hal;

#[rtfm::app(device = lpc81x_hal)]
const APP: () = {
    static mut GPIO17: hal::pins::pin::Pin17<hal::pins::mode::DigitalOutput> = ();

    #[init]
    fn init() -> init::LateResources {
        let cp = core;
        let p = device;

        let pin = p.pins.gpio17.to_digital_output(true);

        // Arrange for the SysTick interrupt to fire every 12,000,000 cycles.
        // (At the MCU's default clock rate, that's once per second.)
        let mut syst = cp.SYST;
        syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
        syst.set_reload(12_000_000);
        syst.clear_current();
        syst.enable_counter();
        syst.enable_interrupt();

        init::LateResources { GPIO17: pin }
    }

    #[exception(resources = [GPIO17])]
    fn SysTick() {
        use embedded_hal::digital::v2::OutputPin;
        resources.GPIO17.set_high().unwrap();
    }
};
