#![no_std]
#![feature(never_type)]

pub extern crate lpc81x_pac as lpc81x;
pub use lpc81x::Interrupt;
pub use lpc81x::NVIC_PRIO_BITS;

pub mod pins;

static mut PAC_PERIPHERALS: Option<lpc81x::Peripherals> = None;

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
}

impl Peripherals {
    fn new() -> Self {
        Self {
            pins: pins::Pins::new(),
            pin_inputs: pins::PinInputs::new(),
        }
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
            Some(p) => {
                unsafe { PAC_PERIPHERALS = Some(p) }
                Some(Self::new())
            }
            None => None,
        }
    }

    pub unsafe fn steal() -> Self {
        Self::new()
    }

    /// Consumes the HAL-level peripherals to unwrap the PAC-level
    /// peripherhals.
    pub fn release_pac(self) -> lpc81x::Peripherals {
        unsafe { self.steal_pac() }
    }

    /// Returns the PAC-level peripherals while leaving the caller with ownership
    /// of the HAL-level peripherals too.
    ///
    /// Although the caller will still have ownership of the HAL-level
    /// peripherals, any subsequent actions taken on them will panic.
    pub unsafe fn steal_pac(&self) -> lpc81x::Peripherals {
        core::mem::replace(&mut PAC_PERIPHERALS, None).unwrap()
    }
}
