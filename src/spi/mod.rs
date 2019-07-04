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
                    // Take the device out of reset first
                    (*syscon)
                        .presetctrl
                        .modify(|_, w| w.$resetctrlfield().bit(enabled));
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
                    })
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
                    })
                }
            }
        }

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
            type Error = !;

            fn send(&mut self, word: W) -> Result<(), nb::Error<!>> {
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
                            .bits(W::LEN)
                    });
                };
                Ok(())
            }

            fn read(&mut self) -> Result<W, nb::Error<!>> {
                let periph = lpc81x_pac::$typename::ptr();
                let stat = unsafe { (*periph).stat.read() };
                if stat.rxrdy().bit_is_clear() {
                    return Err(nb::Error::WouldBlock);
                }
                let raw = unsafe { (*periph).rxdat.read().rxdat().bits() };
                Ok(W::from_received(raw & W::MASK))
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
