//! Illustrates using lpc81x-hal with "Realtime for the Masses" (RTFM).

#![no_std]
#![no_main]
#![feature(asm)]

extern crate cortex_m_rt;
extern crate panic_halt;

use cortex_m_rt::entry;
use lpc81x_hal as hal;
use lpc81x_pac::interrupt;

const DS3231_ADDR: u8 = 0b1101000;

#[entry]
fn main() -> ! {
    // This example demonstrates using the HAL SPI traits to read the date
    // and time from a DS3231 real-time clock IC via its I2C interface.
    //
    // This example assumes the following wiring:
    // - SCL on pin 11
    // - SDA on pin 10
    // - SQW on pin 6
    //
    // We're also expecting to find some leds on pins 7, 17, and 16. On the
    // LPC812-MAX development board these are the red, green, and blue
    // components (respectively) of the on-board RGB LED, wired such that
    // the LEDs are illumnated when the output is driven low.

    cortex_m::interrupt::disable();

    let cp = cortex_m::Peripherals::take().unwrap();
    let p = hal::Peripherals::take().unwrap();

    let pins = p.pins;

    let mut i2c = p.i2c.activate(pins.gpio11, pins.gpio10).enable_host_mode();

    let mut led0 = pins.gpio7.to_digital_output(true);
    let mut led1 = pins.gpio17.to_digital_output(true);
    let mut led2 = pins.gpio16.to_digital_output(true);

    {
        use embedded_hal::blocking::i2c::Write;
        // Bit 2 below is the most important thing: we're turning off INTCN so
        // that the INT/SQW pin will generate a square wave rather than
        // an alarm interrupt.
        // We're also setting bits 3 and 4 to zero, which makes the square
        // wave output be 1Hz, thus generating an edge every half-second.
        // Everything else here is just set to the default values.
        i2c.write(DS3231_ADDR, &[0x0eu8, 0b00000000u8]).unwrap();
    }

    let pinint = p.pin_interrupts.activate();
    let pinint0 = pinint.int0.edge_triggered(pins.gpio6);
    pinint0.enable(true, true);

    unsafe {
        cortex_m::interrupt::enable();
    }

    loop {
        use embedded_hal::blocking::i2c::WriteRead;
        use embedded_hal::digital::v2::OutputPin;
        let mut result: [u8; 7] = [0u8; 7];
        i2c.write_read(DS3231_ADDR, &[0u8], &mut result[..])
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

#[interrupt]
fn PININT0() {
    // We don't actually do anything in here. We're just using this interrupt
    // to make our main loop's wfi return every half-second.
    let periph = lpc81x_pac::PIN_INT::ptr();
    unsafe {
        // Un-pend the interrupt so that we can make progress in our loop.
        (*periph).ist.write(|w| {
            w.pstat().bits(0b00000001) // Clear pinint1
        });
    }
}
