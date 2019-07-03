//! Interface to the SPI peripherals.

use crate::pins;
use core::marker::PhantomData;

pub mod mode;
pub mod word;

macro_rules! spi_device {
    ($typename:ident, {
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
            fn select_ssel(pin: u8) {
                let swm = lpc81x_pac::SWM::ptr();
                unsafe { (*swm).$sselassign.modify(|_, w| w.$sselfield().bits(pin)) }
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
            ) -> $typename<
                mode::Host,
                pins::mode::Assigned<SCLK>,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
            > {
                Self::select_sclk(SCLK::NUMBER);
                unused(sclk);
                $typename::new()
            }

            /// Consumes the inactive SPI bus and returns it with device mode enabled,
            /// using the given pin for SCLK.
            pub fn activate_as_device<SCLK: pins::InputPin>(
                self,
                sclk: SCLK,
            ) -> $typename<
                mode::Device,
                pins::mode::Assigned<SCLK>,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
                pins::mode::Unassigned,
            > {
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

            fn read(&mut self) -> Result<W, nb::Error<!>> {
                panic!("not implemented");
            }

            fn send(&mut self, word: W) -> Result<(), nb::Error<!>> {
                unused(word);
                panic!("not implemented");
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
            ) -> $typename<MODE, SCLK, MOSI, MISO, pins::mode::Assigned<SSEL>> {
                Self::select_ssel(SSEL::NUMBER);
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
                Self::select_ssel(pins::PINASSIGN_NOTHING);
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
                Self::select_sclk(pins::PINASSIGN_NOTHING);
                ($typename::new(), pin_type_as_is())
            }
        }
    };
}

spi_device!(SPI0, {
    SCLK: (pinassign3, spi0_sck_io),
    MOSI: (pinassign4, spi0_mosi_io),
    MISO: (pinassign4, spi0_miso_io),
    SSEL: (pinassign4, spi0_ssel_io)
});
spi_device!(SPI1, {
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
