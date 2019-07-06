//! Illustrates using lpc81x-hal with "Realtime for the Masses" (RTFM).

#![no_std]
#![no_main]
#![feature(asm)]

extern crate cortex_m_rt;
extern crate panic_halt;

use lpc81x_hal as hal;

#[cortex_m_rt::entry]
fn main() -> ! {
    // This example demonstrates using the HAL SPI traits to read the date
    // and time from a DS3231 real-time clock IC via its I2C interface.
    //
    // This example assumes the following wiring:
    // - SCL on pin 11
    // - SDA on pin 10
    //
    // We're also expecting to find some leds on pins 7, 17, and 16. On the
    // LPC812-MAX development board these are the red, green, and blue
    // components (respectively) of the on-board RGB LED, wired such that
    // the LEDs are illumnated when the output is driven low.

    let cp = cortex_m::Peripherals::take().unwrap();
    let p = hal::Peripherals::take().unwrap();

    let pins = p.pins;

    let mut i2c = p.i2c.activate(pins.gpio11, pins.gpio10).enable_host_mode();

    let mut led0 = pins.gpio7.to_digital_output(true);
    let mut led1 = pins.gpio17.to_digital_output(true);
    let mut led2 = pins.gpio16.to_digital_output(true);

    // Arrange for the SysTick interrupt to fire every 12,000,000 cycles.
    // (At the MCU's default clock rate, that's once per second.)
    // (If you have the SQW pin from the DS3231 connected to a pin then
    // it would likely be better to let the RTC chip report its own idea
    // of elapsing seconds, but to keep the wiring simpler for this example
    // we'll just use the internal clock.)
    let mut syst = cp.SYST;
    syst.set_clock_source(cortex_m::peripheral::syst::SystClkSource::Core);
    syst.set_reload(12_000_000);
    syst.clear_current();
    syst.enable_counter();
    syst.enable_interrupt();

    loop {
        use embedded_hal::blocking::i2c::WriteRead;
        use embedded_hal::digital::v2::OutputPin;
        let mut result: [u8; 7] = [0u8; 7];
        i2c.write_read(0b1101000u8, &[0u8], &mut result[..])
            .unwrap();

        let seconds = result[0];
        if (seconds & 0b1) != 0 {
            led0.set_low().unwrap();
        } else {
            led0.set_high().unwrap();
        }
        if (seconds & 0b10) != 0 {
            led1.set_low().unwrap();
        } else {
            led1.set_high().unwrap();
        }
        if (seconds & 0b100) != 0 {
            led2.set_low().unwrap();
        } else {
            led2.set_high().unwrap();
        }

        cortex_m::asm::wfi();
    }
}

#[cortex_m_rt::exception]
fn SysTick() {
    // We don't actually do anything in here. We're just using SysTick to
    // make our main loop's wfi return every second.
}
