//! Interface to the SPI peripherals.

use crate::pins;
use core::marker::PhantomData;

pub mod mode;
pub mod word;

pub struct SPI0<MODE, SCLK, MOSI, MISO, SSEL>
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

impl<MODE, SCLK, MOSI, MISO, SSEL> SPI0<MODE, SCLK, MOSI, MISO, SSEL>
where
    MODE: Mode,
    SCLK: pins::PinAssignment,
    MOSI: pins::PinAssignment,
    MISO: pins::PinAssignment,
    SSEL: pins::PinAssignment,
{
    pub(crate) fn new() -> Self {
        Self {
            mode: PhantomData,
            sclk: PhantomData,
            mosi: PhantomData,
            miso: PhantomData,
            ssel: PhantomData,
        }
    }
}

impl<MODE, SCLK, MOSI, MISO, SSEL> !Sync for SPI0<MODE, SCLK, MOSI, MISO, SSEL> {}

/* ******************************
    METHODS FOR INACTIVE MODE
****************************** */

impl
    SPI0<
        mode::Inactive,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
    >
{
    /// Consumes the inactive SPI bus and returns it with host mode enabled,
    /// using the given pin for SCLK.
    pub fn activate_as_host<SCLK: pins::Pin + pins::UnassignedPin>(
        self,
        sclk: SCLK,
    ) -> SPI0<
        mode::Host,
        pins::mode::Assigned<SCLK>,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
    > {
        unused(sclk);
        panic!("not implemented");
    }

    /// Consumes the inactive SPI bus and returns it with device mode enabled,
    /// using the given pin for SCLK.
    pub fn activate_as_device<SCLK: pins::Pin + pins::InputPin>(
        self,
        sclk: SCLK,
    ) -> SPI0<
        mode::Host,
        pins::mode::Assigned<SCLK>,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
        pins::mode::Unassigned,
    > {
        unused(sclk);
        panic!("not implemented");
    }
}

/* ******************************
     METHODS FOR HOST MODE
****************************** */

impl<SCLK: pins::PinAssignment, SSEL: pins::PinAssignment>
    SPI0<mode::Host, SCLK, pins::mode::Unassigned, pins::mode::Unassigned, SSEL>
{
    pub fn with_data_pins<
        MOSI: pins::Pin + pins::UnassignedPin,
        MISO: pins::Pin + pins::InputPin,
    >(
        self,
        mosi: MOSI,
        miso: MISO,
    ) -> SPI0<mode::Host, SCLK, pins::mode::Assigned<MOSI>, pins::mode::Assigned<MISO>, SSEL> {
        self.with_mosi(mosi).with_miso(miso)
    }
}

impl<SCLK: pins::PinAssignment, MISO: pins::PinAssignment, SSEL: pins::PinAssignment>
    SPI0<mode::Host, SCLK, pins::mode::Unassigned, MISO, SSEL>
{
    pub fn with_mosi<MOSI: pins::Pin + pins::UnassignedPin>(
        self,
        mosi: MOSI,
    ) -> SPI0<mode::Host, SCLK, pins::mode::Assigned<MOSI>, MISO, SSEL> {
        unused(mosi);
        panic!("not implemented");
    }
}

impl<SCLK: pins::PinAssignment, MOSI: pins::PinAssignment, SSEL: pins::PinAssignment>
    SPI0<mode::Host, SCLK, MOSI, pins::mode::Unassigned, SSEL>
{
    pub fn with_miso<MISO: pins::Pin + pins::InputPin>(
        self,
        miso: MISO,
    ) -> SPI0<mode::Host, SCLK, MOSI, pins::mode::Assigned<MISO>, SSEL> {
        unused(miso);
        panic!("not implemented");
    }
}

impl<SCLK: pins::PinAssignment, MOSI: pins::PinAssignment, MISO: pins::PinAssignment>
    SPI0<mode::Host, SCLK, MOSI, MISO, pins::mode::Unassigned>
{
    pub fn with_ssel<SSEL: pins::Pin + pins::UnassignedPin>(
        self,
        ssel: SSEL,
    ) -> SPI0<mode::Host, SCLK, MOSI, MISO, pins::mode::Assigned<SSEL>> {
        unused(ssel);
        panic!("not implemented");
    }
}


impl<W, SCLK, MOSI, MISO, SSEL> embedded_hal::spi::FullDuplex<W> for
    SPI0<mode::Host, SCLK, MOSI, MISO, SSEL>
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

impl<SCLK: pins::PinAssignment, SSEL: pins::PinAssignment>
    SPI0<mode::Device, SCLK, pins::mode::Unassigned, pins::mode::Unassigned, SSEL>
{
    pub fn with_data_pins<
        MOSI: pins::Pin + pins::InputPin,
        MISO: pins::Pin + pins::UnassignedPin,
    >(
        self,
        mosi: MOSI,
        miso: MISO,
    ) -> SPI0<mode::Device, SCLK, pins::mode::Assigned<MOSI>, pins::mode::Assigned<MISO>, SSEL>
    {
        self.with_mosi(mosi).with_miso(miso)
    }
}

impl<SCLK: pins::PinAssignment, MISO: pins::PinAssignment, SSEL: pins::PinAssignment>
    SPI0<mode::Device, SCLK, pins::mode::Unassigned, MISO, SSEL>
{
    pub fn with_mosi<MOSI: pins::Pin + pins::InputPin>(
        self,
        mosi: MOSI,
    ) -> SPI0<mode::Device, SCLK, pins::mode::Assigned<MOSI>, MISO, SSEL> {
        unused(mosi);
        panic!("not implemented");
    }
}

impl<SCLK: pins::PinAssignment, MOSI: pins::PinAssignment, SSEL: pins::PinAssignment>
    SPI0<mode::Device, SCLK, MOSI, pins::mode::Unassigned, SSEL>
{
    pub fn with_miso<MISO: pins::Pin + pins::UnassignedPin>(
        self,
        miso: MISO,
    ) -> SPI0<mode::Device, SCLK, MOSI, pins::mode::Assigned<MISO>, SSEL> {
        unused(miso);
        panic!("not implemented");
    }
}

impl<SCLK: pins::PinAssignment, MOSI: pins::PinAssignment, MISO: pins::PinAssignment>
    SPI0<mode::Device, SCLK, MOSI, MISO, pins::mode::Unassigned>
{
    pub fn with_ssel<SSEL: pins::Pin + pins::InputPin>(
        self,
        ssel: SSEL,
    ) -> SPI0<mode::Device, SCLK, MOSI, MISO, pins::mode::Assigned<SSEL>> {
        unused(ssel);
        panic!("not implemented");
    }
}

/* ******************************
   METHODS FOR ANY ACTIVE MODE
****************************** */

impl<
        MODE: Mode,
        SCLK: pins::PinAssignment,
        MOSI: pins::Pin,
        MISO: pins::Pin,
        SSEL: pins::PinAssignment,
    > SPI0<MODE, SCLK, pins::mode::Assigned<MOSI>, pins::mode::Assigned<MISO>, SSEL>
{
    pub fn release_data_pins(
        self,
    ) -> (
        SPI0<MODE, SCLK, pins::mode::Unassigned, pins::mode::Unassigned, SSEL>,
        MOSI,
        MISO,
    ) {
        let (a, mosi) = self.release_mosi();
        let (b, miso) = a.release_miso();
        (b, mosi, miso)
    }
}

impl<
        MODE: Mode,
        SCLK: pins::PinAssignment,
        MOSI: pins::Pin,
        MISO: pins::PinAssignment,
        SSEL: pins::PinAssignment,
    > SPI0<MODE, SCLK, pins::mode::Assigned<MOSI>, MISO, SSEL>
{
    pub fn release_mosi(self) -> (SPI0<MODE, SCLK, pins::mode::Unassigned, MISO, SSEL>, MOSI) {
        panic!("not implemented");
    }
}

impl<
        MODE: Mode,
        SCLK: pins::PinAssignment,
        MOSI: pins::PinAssignment,
        MISO: pins::Pin,
        SSEL: pins::PinAssignment,
    > SPI0<MODE, SCLK, MOSI, pins::mode::Assigned<MISO>, SSEL>
{
    pub fn release_miso(self) -> (SPI0<MODE, SCLK, MOSI, pins::mode::Unassigned, SSEL>, MISO) {
        panic!("not implemented");
    }
}

impl<
        MODE: Mode,
        SCLK: pins::PinAssignment,
        MOSI: pins::PinAssignment,
        MISO: pins::PinAssignment,
        SSEL: pins::Pin,
    > SPI0<MODE, SCLK, MOSI, MISO, pins::mode::Assigned<SSEL>>
{
    pub fn release_ssel(self) -> (SPI0<MODE, SCLK, MOSI, MISO, pins::mode::Unassigned>, SSEL) {
        panic!("not implemented");
    }
}

impl<MODE: Mode, SCLK: pins::Pin>
    SPI0<
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
        SPI0<
            mode::Inactive,
            pins::mode::Unassigned,
            pins::mode::Unassigned,
            pins::mode::Unassigned,
            pins::mode::Unassigned,
        >,
        SCLK,
    ) {
        panic!("not implemented");
    }
}

// Represents SPI modes.
//
// Can be safely implemented only by types in this crate.
pub unsafe trait Mode {}

fn unused<T>(_v: T) {}
