//! Interface to the I2C peripheral.

use crate::pins;
use core::marker::PhantomData;

/// Represents the I2C peripheral.
///
/// The I2C peripheral starts in an inactive state, not connected to
/// any pins. To use it, call `activate` to activate the peripheral and assign
/// it external pins for the SCL and SDA signals.
pub struct I2C<SCL, SDA>
where
    SCL: pins::PinAssignment,
    SDA: pins::PinAssignment,
{
    scl: PhantomData<SCL>,
    sda: PhantomData<SDA>,
}

impl<SCL, SDA> I2C<SCL, SDA>
where
    SCL: pins::PinAssignment,
    SDA: pins::PinAssignment,
{
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            scl: PhantomData,
            sda: PhantomData,
        }
    }

    #[inline(always)]
    fn select_scl(pin: u8) {
        let swm = lpc81x_pac::SWM::ptr();
        unsafe { (*swm).pinassign8.modify(|_, w| w.i2c_scl_io().bits(pin)) }
    }

    #[inline(always)]
    fn select_sda(pin: u8) {
        let swm = lpc81x_pac::SWM::ptr();
        unsafe { (*swm).pinassign7.modify(|_, w| w.i2c_sda_io().bits(pin)) }
    }

    #[inline(always)]
    fn set_enabled(enabled: bool) {
        let syscon = lpc81x_pac::SYSCON::ptr();
        let periph = lpc81x_pac::I2C::ptr();
        unsafe {
            (*syscon)
                .presetctrl
                .modify(|_, w| w.i2c_rst_n().bit(enabled));
            cortex_m::asm::dsb();
            /*(*periph).cfg.modify(|_, w| {
                if enabled {
                    if host {
                        w.msten().bit(enabled)
                    } else {
                        w.slven().bit(enabled)
                    }
                } else {
                    w.msten().bit(false).slven().bit(false)
                }
            });*/
        }
    }

    #[inline(always)]
    fn set_i2c_clock(active: bool) {
        let syscon = lpc81x_pac::SYSCON::ptr();
        unsafe {
            (*syscon).sysahbclkctrl.modify(|_, w| {
                if active {
                    w.i2c().enable()
                } else {
                    w.i2c().disable()
                }
            });
        }
        cortex_m::asm::dsb();
    }
}

impl I2C<pins::mode::Unassigned, pins::mode::Unassigned> {
    /// Consumes the inactive I2C bus and returns it with host mode enabled,
    /// using the given pins for SCL and SDA.
    ///
    /// Only pins 10 and 11 (in either order) can provide fully I2C-compliant
    /// behavior, but other pins can be used with some caveats. See the LPC81x
    /// user manual for more information and caveats.
    pub fn activate<SCL: pins::UnassignedPin, SDA: pins::UnassignedPin>(
        self,
        scl: SCL,
        sda: SDA,
    ) -> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>> {
        Self::set_i2c_clock(true);
        Self::set_enabled(true);
        Self::select_scl(SCL::NUMBER);
        Self::select_sda(SDA::NUMBER);
        unused(scl);
        unused(sda);
        I2C::new()
    }
}

impl<SCL: pins::Pin, SDA: pins::Pin> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>> {
    /// Consumes the active I2C bus and returns it deactivated, along with
    /// the now-unused pins that were used for SCL and SDA.
    pub fn deactivate(
        self,
    ) -> (
        I2C<pins::mode::Unassigned, pins::mode::Unassigned>,
        SCL,
        SDA,
    ) {
        Self::set_enabled(false);
        Self::select_scl(pins::PINASSIGN_NOTHING);
        Self::select_sda(pins::PINASSIGN_NOTHING);
        Self::set_i2c_clock(false);
        (I2C::new(), pin_type_as_is(), pin_type_as_is())
    }
}

#[inline(always)]
fn unused<T>(_v: T) {}

// Helper function for creating "instances" of our zero-length pin types
// without needing to state their names, when we're releasing/deactivating
// pins.
#[inline(always)]
fn pin_type_as_is<T: pins::Pin>() -> T {
    // This is safe because our pin types are zero-length anyway, and so
    // "filling them with zeroes" is indistinguishable from properly
    // initializing them.
    unsafe { core::mem::zeroed() }
}
