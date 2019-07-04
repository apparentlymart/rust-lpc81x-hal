//! Illustrates using lpc81x-hal with "Realtime for the Masses" (RTFM).

#![no_std]
#![no_main]
#![feature(asm)]

extern crate cortex_m_rt;
extern crate panic_halt;

use lpc81x_hal as hal;

#[cortex_m_rt::entry]
fn main() -> ! {
    // This example demonstrates using the HAL SPI traits to configure an SSD1322
    // display driver that is driving a 256x64 pixel display and then rendering
    // a checkerboard pattern on it, assuming it's running in 4-bit grayscale mode.
    //
    // This example assumes the following wiring:
    // - SCLK on pin 12
    // - MOSI on pin 14
    // - ~CS on pin 13 (driven as a plain GPIO pin, not by the SPI peripheral)
    // - D/C on pin 15
    // - Display ~RESET on pin 16

    let p = hal::Peripherals::take().unwrap();

    let pins = p.pins;

    let mut spi = p
        .spi0
        .activate_as_host(
            pins.gpio12,
            hal::spi::cfg::Config {
                sclk_mode: embedded_hal::spi::MODE_0,
                bit_order: hal::spi::cfg::BitOrder::MSBFirst,
            },
        )
        .with_mosi(pins.gpio14);
    let cs = pins.gpio13.to_digital_output(true);
    let dc = pins.gpio15.to_digital_output(false);
    let mut rst = pins.gpio16.to_digital_output(true);

    // Pulse the reset signal to reset the OLED driver chip before we do
    // anything else.
    {
        use embedded_hal::digital::v2::OutputPin;
        rst.set_low().unwrap();
        rst.set_high().unwrap();
    }

    // Send data over SPI at our maximum rate (divide clock by 1).
    // If the main system clock speed were set faster than the default then
    // this would need to be increased.
    spi.set_clock_divider(1);

    let mut driver = SSD1322::new(spi, cs, dc);
    init(&mut driver).unwrap();

    // We'll allocate a buffer to render our checkerboard pattern into, and then
    // stream it over to the display. We don't have enough memory to buffer
    // the whole screen so we'll just cover a few pixels here.
    const WIDTH: usize = 16;
    const HEIGHT: usize = 16;
    const BPP: usize = 4;
    const BUF_SIZE: usize = HEIGHT * WIDTH / (8 / BPP);
    let mut buf: [u8; BUF_SIZE] = [0; BUF_SIZE];
    for y in 0..HEIGHT {
        // each byte represents two pixels
        for x in 0..(WIDTH / 2) {
            buf[(y * WIDTH / 2 + x)] = if y % 2 == 0 { 0xf0 } else { 0x0f };
        }
    }
    driver.cmd_2(0x15, 0x1C, 0x1C + (WIDTH / 8) as u8).unwrap(); // Set column addresses to constrain to our desired with
    driver.cmd_n(0x5c, &mut buf[..]).unwrap();

    loop {
        cortex_m::asm::wfi();
    }
}

struct SSD1322<
    SPI: embedded_hal::blocking::spi::Write<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
    DC: embedded_hal::digital::v2::OutputPin,
> {
    spi: SPI,
    cs: CS,
    dc: DC,
}

impl<SPI, CS, DC> SSD1322<SPI, CS, DC>
where
    SPI: embedded_hal::blocking::spi::Write<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
    DC: embedded_hal::digital::v2::OutputPin,
{
    pub fn new(spi: SPI, cs: CS, dc: DC) -> Self {
        Self {
            spi: spi,
            cs: cs,
            dc: dc,
        }
    }

    // A real-world driver would hopefully provide a higher-level API than
    // this, but the point here is just to illustrate that this implementation
    // only knows about the embedded-hal traits and is decoupled from the
    // specific lpx81x implementations of them.

    pub fn cmd_0(&mut self, cmd: u8) -> Result<(), ()> {
        self.select()?;
        self.command_mode()?;
        self.write_byte(cmd)?;
        self.deselect()
    }

    pub fn cmd_1(&mut self, cmd: u8, a: u8) -> Result<(), ()> {
        self.select()?;
        self.command_mode()?;
        self.write_byte(cmd)?;
        self.data_mode()?;
        self.write_byte(a)?;
        self.deselect()
    }

    pub fn cmd_2(&mut self, cmd: u8, a: u8, b: u8) -> Result<(), ()> {
        self.select()?;
        self.command_mode()?;
        self.write_byte(cmd)?;
        let msg: [u8; 2] = [a, b];
        self.data_mode()?;
        self.write_bytes(&msg[..])?;
        self.deselect()
    }

    pub fn cmd_n(&mut self, cmd: u8, data: &mut [u8]) -> Result<(), ()> {
        self.select()?;
        self.command_mode()?;
        self.write_byte(cmd)?;
        let mut remain = data;
        self.data_mode()?;
        while remain.len() > 0 {
            let len: usize = if remain.len() > 64 { 64 } else { remain.len() };
            let (this, next) = remain.split_at_mut(len);
            self.write_bytes(this)?;
            remain = next;
        }
        self.deselect()
    }

    fn select(&mut self) -> Result<(), ()> {
        match self.cs.set_low() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn deselect(&mut self) -> Result<(), ()> {
        match self.cs.set_high() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn command_mode(&mut self) -> Result<(), ()> {
        match self.dc.set_low() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn data_mode(&mut self) -> Result<(), ()> {
        match self.dc.set_high() {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn write_byte(&mut self, c: u8) -> Result<(), ()> {
        let tmp: [u8; 1] = [c];
        match self.spi.write(&tmp[..]) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }

    fn write_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        match self.spi.write(data) {
            Ok(_) => Ok(()),
            Err(_) => Err(()),
        }
    }
}

fn init<
    SPI: embedded_hal::blocking::spi::Write<u8>,
    CS: embedded_hal::digital::v2::OutputPin,
    DC: embedded_hal::digital::v2::OutputPin,
>(
    drv: &mut SSD1322<SPI, CS, DC>,
) -> Result<(), ()> {
    // These settings are for the NHD-3.12-25664UCY2 display module, and are
    // derived from its datasheet. Other display modules may need different
    // settings.
    drv.cmd_1(0xfd, 0b00010010)?; // Disable command lock
    drv.cmd_0(0xae)?; // Disable display (just during init; we'll enable it again later)
    drv.cmd_2(0x15, 0x1C, 0x5B)?; // Set column address
    drv.cmd_2(0x75, 0x00, 0x3F)?; // Set row address
    drv.cmd_1(0xb3, 0x91)?; // Set display clock
    drv.cmd_1(0xca, 0x3f)?; // Set multiplex ratio
    drv.cmd_1(0xa2, 0x00)?; // Set display offset
    drv.cmd_1(0xa1, 0x00)?; // Set start line
    drv.cmd_2(0xa0, 0b00010100, 0b00010001)?; // Set remap format
    drv.cmd_1(0xb5, 0x00)?; // Turn off all GPIO
    drv.cmd_1(0xab, 0x01)?; // Enable on-board regulator
    drv.cmd_2(0xb4, 0xA0, 0xFD)?; // Set display Enhancements register A
    drv.cmd_1(0xc1, 0x9f)?; // Set contrast current
    drv.cmd_1(0xc7, 0x0f)?; // Set master current
    drv.cmd_0(0xb9)?; // Select linear grayscale table
    drv.cmd_1(0xb1, 0xe2)?; // Set phase length
    drv.cmd_2(0xd1, 0xa2, 0x20)?; // Set display Enhancements register B
    drv.cmd_1(0xbb, 0x1d)?; // Set precharge voltage
    drv.cmd_1(0xb6, 0x08)?; // Set precharge period
    drv.cmd_1(0xbe, 0x07)?; // Set VCOMH
    drv.cmd_0(0xa6)?; // Normal display mode (not "all on", "all off", or inverted)
    drv.cmd_0(0xaf)?; // Enable display (turns power on)

    Ok(())
}

// We set aside a bit of static memory to stash the HardFault exception frame's
// program counter value. This is just a debugging UX thing.
#[no_mangle]
static mut HARDFAULT_PC: u32 = 0xffffffff;

#[cortex_m_rt::exception]
unsafe fn HardFault(ef: &cortex_m_rt::ExceptionFrame) -> ! {
    loop {
        HARDFAULT_PC = ef.pc;
        cortex_m::asm::dsb();
    }
}
