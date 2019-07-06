//! High-level API and HAL implementations for LPC81x microcontrollers.
//!
//! This library wraps the lower-level `lpc81x-pac` to provide a more ergonomic
//! interface to the LPC81x peripherals, including implementations of some of
//! the `embedded-hal` crates so that these peripheraps can be used with
//! platform-agnostic external device drivers in other crates.
//!
//! -----
//!
//! **Warning:** The interfaces in this crate are _not stable_. Any existing
//! API may see breaking changes in subsequent versions until all of the
//! peripherals have basic support and until `embedded-hal` itself is
//! stable enough to commit to compatibility with it.
//!
//! Pin to a specific version in your application to avoid unexpected breakage.
//!
//! -----
//!
//! The core concept in this library is objects representing the I/O pins.
//! LPC81x parts include a switch matrix that allow most integrated peripherals
//! to be assigned to any of the available pins, and so this library models
//! that capability by having the peripheral objects take ownership of the
//! pins they are assigned to. That allows compile-time checks to ensure that
//! only one output function is assigned to each pin, as is required by the
//! switch matrix hardware.
//!
//! The following example shows how a calling application might activate the
//! SPI0 periopheral and switch a GPIO pin to digital output mode in order to
//! serve as a chip-select signal:
//!
//! ```rust
//! // Consume the peripherals singleton. This also consumes the corresponding
//! // singleton from `lpc81x-pac`, so all peripheral access must be via
//! // this object alone.
//! let p = hal::Peripherals::take().unwrap();
//!
//! // Most of the peripheral APIs require pins as arguments. Unassigned pins
//! // initially live in p.pins and can be moved out into other devices as
//! // needed, which automatically configures the switch matrix.
//! let pins = p.pins;
//!
//! // Use GPIO pins 12 and 14 as the SCLK and MOSI signals respectively.
//! // An SCLK pin is always required to activate an SPI peripheral, but
//! // the other signals can be assigned selectively depending on the needs
//! // of the application. Must assign at least MOSI or MISO for anything
//! // useful to happen, though.
//! let spi = p
//!     .spi0
//!     .activate_as_host(
//!         pins.gpio12,
//!         hal::spi::cfg::Config {
//!             sclk_mode: embedded_hal::spi::MODE_0,
//!             bit_order: hal::spi::cfg::BitOrder::MSBFirst,
//!         },
//!     )
//!     .with_mosi(pins.gpio14);
//!
//! // GPIO pin 13 will be the chip select signal. Calling to_digital_output
//! // consumes the unassigned pin and returns an assigned pin that has been
//! // configured to act as a digital output.
//! let cs = pins.gpio13.to_digital_output(true);
//!
//! // Can now pass the "spi" and "cs" objects into any `embedded-hal` device
//! // driver that works with a SPI interface and a chip select signal.
//! let dev = any_driver::Driver::new(spi, cs);
//! ```
//!
//! There are more examples in the `examples` directory within the crate source
//! code. The above example is a cut-down version of the `ssd1322` example.

#![no_std]
#![feature(never_type)]
#![feature(optin_builtin_traits)]

pub extern crate lpc81x_pac as lpc81x;
pub use lpc81x::Interrupt;
pub use lpc81x::NVIC_PRIO_BITS;

pub mod i2c;
pub mod pinint;
pub mod pins;
pub mod spi;

/// Singleton container for the peripherals modeled by this HAL crate.
///
/// All of the functionality of this library begins in the `Peripherals`
/// singleton. The initial content represents the state of the system and
/// peripherals at reset (after the built-in bootloader launches the user
/// program) and these objects can be moved elsewhere to configure the
/// peripherals for a particular application.
///
/// All of the main interface peripherals take pin objects as arguments.
/// Unassigned pins start off in the `pins` field of `Peripheraps`, and can
/// be moved elsewhere to assign them to functions. This interface ensures
/// at compile time that two outputs cannot be sharing the same pin, which is
/// forbidden by the LPC81x "switch matrix" peripheral.
pub struct Peripherals {
    /// The main accessors for the device pins.
    pub pins: pins::Pins,

    /// Alternative accessors for the device pins' digital inputs.
    ///
    /// This is an alternative to `pins` that provides access only to the input
    /// parts, and crucially allows access to the input parts even when
    /// ownership of the members of `pins` have been transferred elsewhere.
    pub pin_inputs: pins::PinInputs,

    pub pin_interrupts: pinint::Inactive,

    /// The first SPI peripheral, initially inactive.
    pub spi0: spi::SPI0<
        spi::mode::Inactive,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
    >,

    /// The second SPI peripheral, initially inactive.
    ///
    /// This device is only present in models LPC812M101JDH16 and
    /// LPC812M101JDH20 (TSSOP packages).
    pub spi1: spi::SPI1<
        spi::mode::Inactive,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
    >,

    // The I2C peripheral, initially inactive.
    pub i2c: i2c::I2C<
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        i2c::mode::Host<i2c::mode::Inactive>,
        i2c::mode::Device<i2c::mode::Inactive>,
        i2c::mode::Monitor<i2c::mode::Inactive>,
    >,
}

impl Peripherals {
    fn new() -> Self {
        Self {
            pins: pins::Pins::new(),
            pin_inputs: pins::PinInputs::new(),
            pin_interrupts: pinint::Inactive::new(),
            spi0: spi::SPI0::new(),
            spi1: spi::SPI1::new(),
            i2c: i2c::I2C::new(),
        }
    }

    // Can be called only from methods on objects that are only accessible
    // (directly or indirectly) from a Peripherals instance.
    fn pac() -> lpc81x::Peripherals {
        unsafe { lpc81x::Peripherals::steal() }
    }

    /// Obtain (only once) the singleton Peripherals object.
    ///
    /// On subsequent calls, returns `None`.
    pub fn take() -> Option<Self> {
        // Here we rely on the singleton implementation of the underlying
        // PAC Peripherals::take to make this safe. It temporarily disables
        // interrupts while managing its own static mut variable to track
        // whether the singleton was already taken.
        match lpc81x::Peripherals::take() {
            Some(_) => Some(Self::new()),
            None => None,
        }
    }

    pub unsafe fn steal() -> Self {
        Self::new()
    }

    /// Returns a value representing the specific LPC8xx model of the current
    /// device, allowing dynamic inspection of some details that vary by model.
    pub fn model(&self) -> Model {
        let syscon = lpc81x_pac::SYSCON::ptr();
        let raw = unsafe { (*syscon).device_id.read().deviceid().bits() };
        match raw {
            0x00008100 => Model::LPC810M021FN8,
            0x00008110 => Model::LPC811M001JDH16,
            0x00008120 => Model::LPC812M101JDH16,
            0x00008121 => Model::LPC812M101JD20,
            0x00008122 => Model::LPC812M101JDH20,
            0x00008123 => Model::LPC812M101JTB16,
            _ => panic!("unsupported model"),
        }
    }

    /// Consumes the HAL-level peripherals to unwrap the PAC-level
    /// peripherhals.
    pub fn release_pac(self) -> lpc81x::Peripherals {
        unsafe { self.steal_pac() }
    }

    /// Returns the PAC-level peripherals while leaving the caller with ownership
    /// of the HAL-level peripherals too.
    ///
    /// This is unsafe because the raw PAC peripherals API may be used to
    /// reconfigure the hardware in a way that the HAL isn't aware of.
    pub unsafe fn steal_pac(&self) -> lpc81x::Peripherals {
        Self::pac()
    }
}

/// Identifies a particular model of LPC8xx device.
pub enum Model {
    LPC810M021FN8,
    LPC811M001JDH16,
    LPC812M101JDH16,
    LPC812M101JD20,
    LPC812M101JDH20,
    LPC812M101JTB16,
}

impl Model {
    /// Returns true if the given pin is available for this model.
    ///
    /// This library does not prevent using pins that are not available on
    /// a particular device model or package, but an application intended to
    /// be portable can use this to adapt to different pin subsets.
    pub fn has_pin<P: pins::Pin>(&self, pin: &P) -> bool {
        unused(pin);
        match self {
            Model::LPC810M021FN8 => P::NUMBER <= 5,
            _ => true,
        }
    }

    /// Returns true if the second SPI device (SPI1) is available for this
    /// model.
    ///
    /// This library does not prevent using SPI1 on devices where it is
    /// unavailable. An application intended to be portable can use this to
    /// detect when SPI1 is unavailable and perform some sort of graceful
    /// fallback, such as using a "bit-bang" SPI implementation.
    pub fn has_spi1(&self) -> bool {
        match self {
            Model::LPC812M101JDH16 => true,
            Model::LPC812M101JDH20 => true,
            _ => false,
        }
    }
}

#[inline(always)]
fn unused<T>(_v: T) {}
