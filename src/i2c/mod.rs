//! Interface to the I2C peripheral.

use crate::pins;
use core::marker::PhantomData;

pub mod mode;

/// Represents the I2C peripheral.
///
/// The I2C peripheral starts in an inactive state, not connected to
/// any pins. To use it, call `activate` to activate the peripheral and assign
/// it external pins for the SCL and SDA signals.
///
/// The I2C peripheral has three modes that can each be independently activated:
///
/// * Host mode: initiates communication with devices on the bus; enable with `enable_host_mode`.
/// * Device mode: responds to communication requests from hosts on the bus; enable with `enable_device_mdoe`.
/// * Monitor mode: monitors communications on the bus without transmitting anything; enable with `enable_monitor_mode`.
///
/// With no modes activated, the I2C peripheral is powered but cannot transmit
/// or recieve any data.
#[derive(Debug)]
pub struct I2C<SCL, SDA, HS, DS, MS>
where
    SCL: pins::PinAssignment,
    SDA: pins::PinAssignment,
    HS: mode::HostStatus,
    DS: mode::DeviceStatus,
    MS: mode::MonitorStatus,
{
    scl: PhantomData<SCL>,
    sda: PhantomData<SDA>,
    modes: PhantomData<(HS, DS, MS)>,
}

impl<SCL, SDA, HS, DS, MS> !Sync for I2C<SCL, SDA, HS, DS, MS> {}

impl<SCL, SDA, HS, DS, MS> I2C<SCL, SDA, HS, DS, MS>
where
    SCL: pins::PinAssignment,
    SDA: pins::PinAssignment,
    HS: mode::HostStatus,
    DS: mode::DeviceStatus,
    MS: mode::MonitorStatus,
{
    #[inline(always)]
    pub(crate) fn new() -> Self {
        Self {
            scl: PhantomData,
            sda: PhantomData,
            modes: PhantomData,
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
        unsafe {
            (*syscon)
                .presetctrl
                .modify(|_, w| w.i2c_rst_n().bit(enabled));
            cortex_m::asm::dsb();
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

    #[inline(always)]
    fn addr_mode(addr: u8, write: bool) -> u8 {
        addr << 1 | if write { 0 } else { 1 }
    }
}

impl
    I2C<
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        mode::HostInactive,
        mode::DeviceInactive,
        mode::MonitorInactive,
    >
{
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
    ) -> I2C<
        pins::mode::Assigned<SCL>,
        pins::mode::Assigned<SDA>,
        mode::HostInactive,
        mode::DeviceInactive,
        mode::MonitorInactive,
    > {
        Self::set_i2c_clock(true);
        Self::set_enabled(true);
        Self::select_scl(SCL::NUMBER);
        Self::select_sda(SDA::NUMBER);
        unused(scl);
        unused(sda);
        I2C::new()
    }
}

impl<SCL, SDA, DS, MS>
    I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, mode::HostInactive, DS, MS>
where
    SCL: pins::Pin,
    SDA: pins::Pin,
    DS: mode::DeviceStatus,
    MS: mode::MonitorStatus,
{
    /// Consumes the active I2C bus and returns it with host mode activated,
    /// and thus with [the host-mode-only methods](#host-mode-methods) available.
    pub fn enable_host_mode(
        &self,
    ) -> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, mode::HostActive, DS, MS> {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe {
            (*periph).cfg.modify(|_, w| w.msten().bit(true));
        }
        I2C::new()
    }
}

impl<SCL, SDA, HS, MS>
    I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, mode::DeviceInactive, MS>
where
    SCL: pins::Pin,
    SDA: pins::Pin,
    HS: mode::HostStatus,
    MS: mode::MonitorStatus,
{
    /// Consumes the active I2C bus and returns it with device mode activated,
    /// and thus with [the device-mode-only methods](#device-mode-methods) available.
    pub fn enable_device_mode(
        &self,
    ) -> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, mode::DeviceActive, MS> {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe {
            (*periph).cfg.modify(|_, w| w.slven().bit(true));
        }
        I2C::new()
    }
}

impl<SCL, SDA, HS, DS>
    I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, DS, mode::MonitorInactive>
where
    SCL: pins::Pin,
    SDA: pins::Pin,
    HS: mode::HostStatus,
    DS: mode::DeviceStatus,
{
    /// Consumes the active I2C bus and returns it with monitor mode activated,
    /// and thus with [the monitor-mode-only methods](#monitor-mode-methods) available.
    pub fn enable_monitor_mode(
        &self,
    ) -> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, DS, mode::MonitorActive>
    {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe {
            (*periph).cfg.modify(|_, w| w.monen().bit(true));
        }
        I2C::new()
    }
}

/// ## Host mode methods
///
/// These methods are available only once host mode is active.
impl<SCL, SDA, DS, MS>
    I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, mode::HostActive, DS, MS>
where
    SCL: pins::Pin,
    SDA: pins::Pin,
    DS: mode::DeviceStatus,
    MS: mode::MonitorStatus,
{
    /// Consumes the active I2C bus and returns it with host mode deactivated.
    pub fn disable_host_mode(
        &self,
    ) -> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, mode::HostInactive, DS, MS> {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe {
            (*periph).cfg.modify(|_, w| w.msten().bit(false));
        }
        I2C::new()
    }

    #[inline(always)]
    fn block_for_host_mode_pending() -> Result<(), HostError> {
        let periph = lpc81x_pac::I2C::ptr();

        loop {
            let r = unsafe { (*periph).stat.read() };
            if r.mstarbloss().bit_is_set() {
                return Err(HostError::ArbitrationLoss);
            }
            if r.mstststperr().bit_is_set() {
                return Err(HostError::StartStop);
            }
            if r.mstpending().bit_is_set() {
                return Ok(());
            }
        }
    }

    #[inline(always)]
    fn set_host_mode_data(d: u8) {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe { (*periph).mstdat.write(|w| w.data().bits(d)) }
    }

    #[inline(always)]
    fn get_host_mode_data() -> u8 {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe { (*periph).mstdat.read().data().bits() }
    }

    #[inline(always)]
    fn host_mode_start() {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe { (*periph).mstctl.write(|w| w.mststart().set_bit()) }
    }

    #[inline(always)]
    fn host_mode_continue() {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe { (*periph).mstctl.write(|w| w.mstcontinue().set_bit()) }
    }

    #[inline(always)]
    fn host_mode_stop() {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe { (*periph).mstctl.write(|w| w.mststop().set_bit()) }
    }
}

impl<SCL, SDA, DS, MS> embedded_hal::blocking::i2c::WriteRead
    for I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, mode::HostActive, DS, MS>
where
    SCL: pins::Pin,
    SDA: pins::Pin,
    DS: mode::DeviceStatus,
    MS: mode::MonitorStatus,
{
    type Error = HostError;

    fn write_read(
        &mut self,
        address: u8,
        bytes: &[u8],
        buffer: &mut [u8],
    ) -> Result<(), Self::Error> {
        let addr_wr = Self::addr_mode(address, true);
        let addr_rd = Self::addr_mode(address, false);

        Self::block_for_host_mode_pending()?;
        Self::set_host_mode_data(addr_wr);
        Self::host_mode_start();

        for c in bytes {
            Self::block_for_host_mode_pending()?;
            Self::set_host_mode_data(*c);
            Self::host_mode_continue();
        }

        Self::block_for_host_mode_pending()?;
        Self::set_host_mode_data(addr_rd);
        Self::host_mode_start();

        for (i, c) in buffer.iter_mut().enumerate() {
            if i > 0 {
                Self::block_for_host_mode_pending()?;
                Self::host_mode_continue();
            }
            Self::block_for_host_mode_pending()?;
            *c = Self::get_host_mode_data();
        }

        Self::host_mode_stop();

        Ok(())
    }
}

/// ## Device mode methods
///
/// These methods are available only once device mode is active.
impl<SCL, SDA, HS, MS>
    I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, mode::DeviceActive, MS>
where
    SCL: pins::Pin,
    SDA: pins::Pin,
    HS: mode::HostStatus,
    MS: mode::MonitorStatus,
{
    /// Consumes the active I2C bus and returns it with device mode deactivated.
    pub fn disable_device_mode(
        &self,
    ) -> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, mode::DeviceInactive, MS>
    {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe {
            (*periph).cfg.modify(|_, w| w.slven().bit(false));
        }
        I2C::new()
    }
}

/// ## Monitor mode methods
///
/// These methods are available only once monitor mode is active.
impl<SCL, SDA, HS, DS>
    I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, DS, mode::MonitorActive>
where
    SCL: pins::Pin,
    SDA: pins::Pin,
    HS: mode::HostStatus,
    DS: mode::DeviceStatus,
{
    /// Consumes the active I2C bus and returns it with monitor mode deactivated.
    pub fn disable_monitor_mode(
        &self,
    ) -> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, DS, mode::MonitorInactive>
    {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe {
            (*periph).cfg.modify(|_, w| w.monen().bit(false));
        }
        I2C::new()
    }
}

impl<SCL, SDA, HS, DS, MS> I2C<pins::mode::Assigned<SCL>, pins::mode::Assigned<SDA>, HS, DS, MS>
where
    SCL: pins::Pin,
    SDA: pins::Pin,
    HS: mode::HostStatus,
    DS: mode::DeviceStatus,
    MS: mode::MonitorStatus,
{
    /// Consumes the active I2C bus and returns it deactivated, along with
    /// the now-unused pins that were used for SCL and SDA.
    pub fn deactivate(
        self,
    ) -> (
        I2C<
            pins::mode::Unassigned,
            pins::mode::Unassigned,
            mode::HostInactive,
            mode::DeviceInactive,
            mode::MonitorInactive,
        >,
        SCL,
        SDA,
    ) {
        let periph = lpc81x_pac::I2C::ptr();
        unsafe {
            (*periph).cfg.write(|w| w); // Set back to the reset value
        }
        Self::set_enabled(false);
        Self::select_scl(pins::PINASSIGN_NOTHING);
        Self::select_sda(pins::PINASSIGN_NOTHING);
        Self::set_i2c_clock(false);
        (I2C::new(), pin_type_as_is(), pin_type_as_is())
    }
}

#[derive(Debug)]
pub enum HostError {
    Request,
    ArbitrationLoss,
    StartStop,
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
