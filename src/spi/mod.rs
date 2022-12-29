//! Interface to the SPI peripherals.

use crate::pins;
use core::marker::PhantomData;

pub mod cfg;
pub mod mode;
pub mod word;

macro_rules! spi_device {
    ($typename:ident, $fieldname:ident, {
        RESETCTRL: $resetctrlfield:ident,
        CLKCTRL: $clkctrlfield:ident,
        SCLK: ($sclkassign:ident, $sclkfield:ident),
        MOSI: ($mosiassign:ident, $mosifield:ident),
        MISO: ($misoassign:ident, $misofield:ident),
        SSEL: ($sselassign:ident, $sselfield:ident)
    }) => {
        /// Represents the SPI peripheral.
        ///
        /// Each SPI peripheral starts in an inactive state, not connected to
        /// any pins. To use it, call either `activate_as_host` or
        /// `activate_as_device` to activate the peripheral and assign it
        /// an external pin for the SCLK signal.
        ///
        /// Once activated, call either `with_data_pins`, `with_mosi`, or
        /// `with_miso` to assign further external pins for the MOSI and MISO
        /// signals, as required.
        ///
        /// An activated SPI peripheral in host mode implements the
        /// `embedded-hal` SPI traits, so you can pass it directly to a device
        /// driver that expects any of these traits.
        pub struct $typename<MODE, SCLK, MOSI, MISO, SSEL>
        where
            MODE: Mode,
            SCLK: pins::PinAssignment,
            MOSI: pins::PinAssignment,
            MISO: pins::PinAssignment,
            SSEL: pins::PinAssignment,
        {
            mode: PhantomData<MODE>,
            sclk: PhantomData<SCLK>,
            mosi: PhantomData<MOSI>,
            miso: PhantomData<MISO>,
            ssel: PhantomData<SSEL>,
        }

        impl<MODE, SCLK, MOSI, MISO, SSEL> $typename<MODE, SCLK, MOSI, MISO, SSEL>
        where
            MODE: Mode,
            SCLK: pins::PinAssignment,
            MOSI: pins::PinAssignment,
            MISO: pins::PinAssignment,
            SSEL: pins::PinAssignment,
        {
            #[inline(always)]
            pub(crate) fn new() -> Self {
                Self {
                    mode: PhantomData,
                    sclk: PhantomData,
                    mosi: PhantomData,
                    miso: PhantomData,
                    ssel: PhantomData,
                }
            }

            #[inline(always)]
            fn select_sclk(pin: u8) {
                let swm = lpc81x_pac::SWM::ptr();
                unsafe { (*swm).$sclkassign.modify(|_, w| w.$sclkfield().bits(pin)) }
            }

            #[inline(always)]
            fn select_mosi(pin: u8) {
                let swm = lpc81x_pac::SWM::ptr();
                unsafe { (*swm).$mosiassign.modify(|_, w| w.$mosifield().bits(pin)) }
            }

            #[inline(always)]
            fn select_miso(pin: u8) {
                let swm = lpc81x_pac::SWM::ptr();
                unsafe { (*swm).$misoassign.modify(|_, w| w.$misofield().bits(pin)) }
            }

            #[inline(always)]
            fn select_ssel(pin: u8, polarity: cfg::Polarity) {
                let swm = lpc81x_pac::SWM::ptr();
                let periph = lpc81x_pac::$typename::ptr();
                unsafe { (*swm).$sselassign.modify(|_, w| w.$sselfield().bits(pin)) }
                unsafe {
                    (*periph).cfg.modify(|_, w| {
                        w.spol().bit(if let cfg::Polarity::ActiveHigh = polarity {
                            true
                        } else {
                            false
                        })
                    })
                }
            }

            #[inline(always)]
            fn set_enabled(enabled: bool, host: bool, cfg: cfg::Config) {
                let syscon = lpc81x_pac::SYSCON::ptr();
                let periph = lpc81x_pac::$typename::ptr();
                unsafe {
                    if enabled {
                        // Take the device out of reset first
                        (*syscon)
                            .presetctrl
                            .modify(|_, w| w.$resetctrlfield().bit(true));
                        cortex_m::asm::dsb();
                    }
                    (*periph).cfg.modify(|_, w| {
                        w.enable()
                            .bit(enabled)
                            .master()
                            .bit(host)
                            .lsbf()
                            .bit(if let cfg::BitOrder::LSBFirst = cfg.bit_order {
                                true
                            } else {
                                false
                            })
                            .cpha()
                            .bit(
                                if let embedded_hal::spi::Phase::CaptureOnSecondTransition =
                                    cfg.sclk_mode.phase
                                {
                                    true
                                } else {
                                    false
                                },
                            )
                            .cpol()
                            .bit(
                                if let embedded_hal::spi::Polarity::IdleHigh =
                                    cfg.sclk_mode.polarity
                                {
                                    true
                                } else {
                                    false
                                },
                            )
                    });
                    if !enabled {
                        cortex_m::asm::dsb();
                        (*syscon)
                            .presetctrl
                            .modify(|_, w| w.$resetctrlfield().bit(false));
                    }
                }
            }

            #[inline(always)]
            fn set_spi_clock(active: bool) {
                let syscon = lpc81x_pac::SYSCON::ptr();
                unsafe {
                    (*syscon).sysahbclkctrl.modify(|_, w| {
                        if active {
                            w.$clkctrlfield().enable()
                        } else {
                            w.$clkctrlfield().disable()
                        }
                    });
                }
                cortex_m::asm::dsb();
            }
        }

        /// An SPI peripheral object represents access to a single system
        /// peripheral, so it's not safe to share it across multiple threads
        /// without some external concurrency control mechanisms.
        impl<MODE, SCLK, MOSI, MISO, SSEL> !Sync for $typename<MODE, SCLK, MOSI, MISO, SSEL> {}

        /* ******************************
            METHODS FOR INACTIVE MODE
        ****************************** */

        impl
            $typename<
                mode::Inactive,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
            >
        {
            /// Consumes the inactive SPI bus and returns it with host mode enabled,
            /// using the given pin for SCLK.
            pub fn activate_as_host<SCLK: pins::UnassignedPin>(
                self,
                sclk: SCLK,
                cfg: cfg::Config,
            ) -> $typename<
                mode::Host,
                pins::mode::Assigned<SCLK>,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
            > {
                Self::set_spi_clock(true);
                Self::set_enabled(true, true, cfg);
                Self::select_sclk(SCLK::NUMBER);
                unused(sclk);
                $typename::new()
            }

            /// Consumes the inactive SPI bus and returns it with device mode enabled,
            /// using the given pin for SCLK.
            pub fn activate_as_device<SCLK: pins::InputPin>(
                self,
                sclk: SCLK,
                cfg: cfg::Config,
            ) -> $typename<
                mode::Device,
                pins::mode::Assigned<SCLK>,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
            > {
                Self::set_spi_clock(true);
                Self::set_enabled(true, false, cfg);
                Self::select_sclk(SCLK::NUMBER);
                unused(sclk);
                $typename::new()
            }
        }

        /* ******************************
            METHODS FOR HOST MODE
        ****************************** */

        impl<W, SCLK, MOSI, MISO, SSEL> embedded_hal::spi::FullDuplex<W>
            for $typename<mode::Host, SCLK, MOSI, MISO, SSEL>
        where
            W: word::Word,
            SCLK: pins::PinAssignment,
            MOSI: pins::PinAssignment,
            MISO: pins::PinAssignment,
            SSEL: pins::PinAssignment,
        {
            type Error = void::Void;

            /// Sends a word (from 1 to 16 bits) over the SPI interface.
            ///
            /// The `embedded-hal` contract calls for chip select to be managed
            /// separately by calling code, so `send` does not automatically
            /// assert and unassert the SSEL signal to delimit the transaction.
            /// When using this trait implementation, implement chip select as
            /// a generic digital output pin instead, because that is what
            /// embedded-hal device drivers expect.
            fn send(&mut self, word: W) -> Result<(), nb::Error<void::Void>> {
                let periph = lpc81x_pac::$typename::ptr();
                let stat = unsafe { (*periph).stat.read() };
                if stat.txrdy().bit_is_clear() {
                    return Err(nb::Error::WouldBlock);
                }
                unsafe {
                    (*periph).txdatctl.write(|w| {
                        w.txdat()
                            .bits(word.value_to_transmit() & W::MASK)
                            .flen()
                            .bits(W::LEN - 1)
                    });
                };
                Ok(())
            }

            /// Reads the word returned from the device after a `send`.
            ///
            /// Calling `read` once for every `send` is mandatory in order to
            /// leave the SPI bus in a correct state for subsequent transfers.
            fn read(&mut self) -> Result<W, nb::Error<void::Void>> {
                let periph = lpc81x_pac::$typename::ptr();
                let stat = unsafe { (*periph).stat.read() };
                if stat.rxrdy().bit_is_clear() {
                    return Err(nb::Error::WouldBlock);
                }
                let raw = unsafe { (*periph).rxdat.read().rxdat().bits() };
                Ok(W::from_received(raw & W::MASK))
            }
        }

        impl<W, SCLK, MOSI, MISO, SSEL> embedded_hal::blocking::spi::write::Default<W>
            for $typename<mode::Host, SCLK, MOSI, MISO, SSEL>
        where
            W: word::Word,
            SCLK: pins::PinAssignment,
            MOSI: pins::PinAssignment,
            MISO: pins::PinAssignment,
            SSEL: pins::PinAssignment,
        {
        }

        impl<W, SCLK, MOSI, MISO, SSEL> embedded_hal::blocking::spi::write_iter::Default<W>
            for $typename<mode::Host, SCLK, MOSI, MISO, SSEL>
        where
            W: word::Word,
            SCLK: pins::PinAssignment,
            MOSI: pins::PinAssignment,
            MISO: pins::PinAssignment,
            SSEL: pins::PinAssignment,
        {
        }

        impl<W, SCLK, MOSI, MISO, SSEL> embedded_hal::blocking::spi::transfer::Default<W>
            for $typename<mode::Host, SCLK, MOSI, MISO, SSEL>
        where
            W: word::Word,
            SCLK: pins::PinAssignment,
            MOSI: pins::PinAssignment,
            MISO: pins::PinAssignment,
            SSEL: pins::PinAssignment,
        {
        }

        impl<
                SCLK: pins::PinAssignment,
                MOSI: pins::PinAssignment,
                MISO: pins::PinAssignment,
                SSEL: pins::PinAssignment,
            > $typename<mode::Host, SCLK, MOSI, MISO, SSEL>
        {
            // Configures the clock divider for the SPI peripheral.
            //
            // The frequency of the SPI clock is the frequency of the system
            // clock divided by the given divisor. The SPI peripheral accepts
            // divisors between 1 and 65536. If the given divisor is not within
            // that range then it will be capped to the closest limit to
            // keep it in range.
            pub fn set_clock_divider(&mut self, div: u32) {
                let mut real_div = div;
                if div < 1 {
                    real_div = 1;
                } else if div > 65536 {
                    real_div = 65536;
                }
                let periph = lpc81x_pac::$typename::ptr();
                unsafe {
                    (*periph)
                        .div
                        .write(|w| w.divval().bits((real_div - 1) as u16))
                }
            }
        }

        /* ******************************
            METHODS FOR DEVICE MODE
        ****************************** */

        /* ******************************
           METHODS FOR ANY ACTIVE MODE
        ****************************** */

        impl<MODE: mode::Active, SCLK: pins::PinAssignment, SSEL: pins::PinAssignment>
            $typename<MODE, SCLK, pins::mode::Unassigned, pins::mode::Unassigned, SSEL>
        {
            /// Assigns pins to both the MOSI and MISO signals at once.
            ///
            /// This is just a convenience shortcut for `.with_mosi(mosi).with_miso(miso)`.
            pub fn with_data_pins<MOSI: pins::UnassignedPin, MISO: pins::UnassignedPin>(
                self,
                mosi: MOSI,
                miso: MISO,
            ) -> $typename<MODE, SCLK, pins::mode::Assigned<MOSI>, pins::mode::Assigned<MISO>, SSEL>
            {
                self.with_mosi(mosi).with_miso(miso)
            }
        }

        impl<
                MODE: mode::Active,
                SCLK: pins::PinAssignment,
                MISO: pins::PinAssignment,
                SSEL: pins::PinAssignment,
            > $typename<MODE, SCLK, pins::mode::Unassigned, MISO, SSEL>
        {
            /// Assigns an unassigned external pin to the MOSI signal.
            pub fn with_mosi<MOSI: pins::UnassignedPin>(
                self,
                mosi: MOSI,
            ) -> $typename<MODE, SCLK, pins::mode::Assigned<MOSI>, MISO, SSEL> {
                Self::select_mosi(MOSI::NUMBER);
                unused(mosi);
                $typename::new()
            }
        }

        impl<
                MODE: mode::Active,
                SCLK: pins::PinAssignment,
                MOSI: pins::PinAssignment,
                SSEL: pins::PinAssignment,
            > $typename<MODE, SCLK, MOSI, pins::mode::Unassigned, SSEL>
        {
            /// Assigns an unassigned external pin to the MISO signal.
            pub fn with_miso<MISO: pins::UnassignedPin>(
                self,
                miso: MISO,
            ) -> $typename<MODE, SCLK, MOSI, pins::mode::Assigned<MISO>, SSEL> {
                Self::select_miso(MISO::NUMBER);
                unused(miso);
                $typename::new()
            }
        }

        impl<
                MODE: mode::Active,
                SCLK: pins::PinAssignment,
                MOSI: pins::PinAssignment,
                MISO: pins::PinAssignment,
            > $typename<MODE, SCLK, MOSI, MISO, pins::mode::Unassigned>
        {
            /// Assigns an unassigned external pin to the SSEL signal.
            pub fn with_ssel<SSEL: pins::UnassignedPin>(
                self,
                ssel: SSEL,
                polarity: cfg::Polarity,
            ) -> $typename<MODE, SCLK, MOSI, MISO, pins::mode::Assigned<SSEL>> {
                Self::select_ssel(SSEL::NUMBER, polarity);
                unused(ssel);
                $typename::new()
            }
        }

        impl<
                MODE: mode::Active,
                SCLK: pins::PinAssignment,
                MOSI: pins::Pin,
                MISO: pins::Pin,
                SSEL: pins::PinAssignment,
            > $typename<MODE, SCLK, pins::mode::Assigned<MOSI>, pins::mode::Assigned<MISO>, SSEL>
        {
            /// Consumes the SPI object and returns a new object with the MOSI
            /// and MISO pins detached.
            ///
            /// Along with that new object, the former MOSI and MISO pins are
            /// also returned in unassigned mode, ready to be assigned to
            /// another function.
            pub fn release_data_pins(
                self,
            ) -> (
                $typename<MODE, SCLK, pins::mode::Unassigned, pins::mode::Unassigned, SSEL>,
                MOSI,
                MISO,
            ) {
                let (a, mosi) = self.release_mosi();
                let (b, miso) = a.release_miso();
                (b, mosi, miso)
            }
        }

        impl<
                MODE: mode::Active,
                SCLK: pins::PinAssignment,
                MOSI: pins::Pin,
                MISO: pins::PinAssignment,
                SSEL: pins::PinAssignment,
            > $typename<MODE, SCLK, pins::mode::Assigned<MOSI>, MISO, SSEL>
        {
            /// Consumes the SPI object and returns a new object with the MOSI
            /// pin detached.
            ///
            /// Along with that new object, the former MOSI pin is also
            /// returned in unassigned mode, ready to be assigned to another
            /// function.
            pub fn release_mosi(
                self,
            ) -> (
                $typename<MODE, SCLK, pins::mode::Unassigned, MISO, SSEL>,
                MOSI,
            ) {
                Self::select_mosi(pins::PINASSIGN_NOTHING);
                ($typename::new(), pin_type_as_is())
            }
        }

        impl<
                MODE: mode::Active,
                SCLK: pins::PinAssignment,
                MOSI: pins::PinAssignment,
                MISO: pins::Pin,
                SSEL: pins::PinAssignment,
            > $typename<MODE, SCLK, MOSI, pins::mode::Assigned<MISO>, SSEL>
        {
            /// Consumes the SPI object and returns a new object with the MISO
            /// pin detached.
            ///
            /// Along with that new object, the former MISO pin is also
            /// returned in unassigned mode, ready to be assigned to another
            /// function.
            pub fn release_miso(
                self,
            ) -> (
                $typename<MODE, SCLK, MOSI, pins::mode::Unassigned, SSEL>,
                MISO,
            ) {
                Self::select_miso(pins::PINASSIGN_NOTHING);
                ($typename::new(), pin_type_as_is())
            }
        }

        impl<
                MODE: mode::Active,
                SCLK: pins::PinAssignment,
                MOSI: pins::PinAssignment,
                MISO: pins::PinAssignment,
                SSEL: pins::Pin,
            > $typename<MODE, SCLK, MOSI, MISO, pins::mode::Assigned<SSEL>>
        {
            /// Consumes the SPI object and returns a new object with the SSEL
            /// pin detached.
            ///
            /// Along with that new object, the former SSEL pin is also
            /// returned in unassigned mode, ready to be assigned to another
            /// function.
            pub fn release_ssel(
                self,
            ) -> (
                $typename<MODE, SCLK, MOSI, MISO, pins::mode::Unassigned>,
                SSEL,
            ) {
                Self::select_ssel(pins::PINASSIGN_NOTHING, cfg::Polarity::ActiveLow);
                ($typename::new(), pin_type_as_is())
            }
        }

        impl<MODE: mode::Active, SCLK: pins::Pin>
            $typename<
                MODE,
                pins::mode::Assigned<SCLK>,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
            >
        {
            /// Consumes the active SPI bus and returns it deactivated, along with
            /// the now-unused pin that was used for SCK.
            ///
            /// This method can be called only when any assigned MOSI, MISO,
            /// and SSEL pins have already been released. For example:
            ///
            /// ```rust
            /// let (spi, mosi, miso) = spi.release_data_pins();
            /// // (for this example, assume the object never had SSEL active)
            /// let (spi, sclk) = spi.deactivate();
            /// ```
            pub fn deactivate(
                self,
            ) -> (
                $typename<
                    mode::Inactive,
                    pins::mode::Unassigned,
                    pins::mode::Unassigned,
                    pins::mode::Unassigned,
                    pins::mode::Unassigned,
                >,
                SCLK,
            ) {
                Self::set_enabled(false, false, cfg::RESET_CONFIG);
                Self::select_sclk(pins::PINASSIGN_NOTHING);
                Self::set_spi_clock(false);
                ($typename::new(), pin_type_as_is())
            }
        }
    };
}

spi_device!(SPI0, spi0, {
    RESETCTRL: spi0_rst_n,
    CLKCTRL: spi0,
    SCLK: (pinassign3, spi0_sck_io),
    MOSI: (pinassign4, spi0_mosi_io),
    MISO: (pinassign4, spi0_miso_io),
    SSEL: (pinassign4, spi0_ssel_io)
});
spi_device!(SPI1, spi1, {
    RESETCTRL: spi1_rst_n,
    CLKCTRL: spi1,
    SCLK: (pinassign4, spi1_sck_io),
    MOSI: (pinassign5, spi1_mosi_io),
    MISO: (pinassign5, spi1_miso_io),
    SSEL: (pinassign5, spi1_ssel_io)
});

// Represents SPI modes.
//
// Can be safely implemented only by types in this crate.
pub unsafe trait Mode {}

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
