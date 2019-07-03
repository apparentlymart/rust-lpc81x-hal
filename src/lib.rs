#![no_std]
#![feature(never_type)]
#![feature(optin_builtin_traits)]

pub extern crate lpc81x_pac as lpc81x;
pub use lpc81x::Interrupt;
pub use lpc81x::NVIC_PRIO_BITS;

pub mod pins;
pub mod spi;

/// Singleton container for the peripherals modeled by this HAL crate.
pub struct Peripherals {
    /// The main accessors for the device pins.
    pub pins: pins::Pins,

    /// Alternative accessors for the device pins' digital inputs.
    ///
    /// This is an alternative to `pins` that provides access only to the input
    /// parts, and crucially allows access to the input parts even when
    /// ownership of the members of `pins` have been transferred elsewhere.
    pub pin_inputs: pins::PinInputs,

    /// The first SPI device, initially inactive.
    pub spi0: spi::SPI0<
        spi::mode::Inactive,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
    >,

    /// The second SPI device, initially inactive.
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
}

impl Peripherals {
    fn new() -> Self {
        Self {
            pins: pins::Pins::new(),
            pin_inputs: pins::PinInputs::new(),
            spi0: spi::SPI0::new(),
            spi1: spi::SPI1::new(),
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

pub enum Model {
    LPC810M021FN8,
    LPC811M001JDH16,
    LPC812M101JDH16,
    LPC812M101JD20,
    LPC812M101JDH20,
    LPC812M101JTB16,
}

impl Model {
    pub fn has_pin<P: pins::Pin>(&self, pin: P) -> bool {
        unused(pin);
        match self {
            Model::LPC810M021FN8 => P::NUMBER <= 5,
            _ => true,
        }
    }

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
