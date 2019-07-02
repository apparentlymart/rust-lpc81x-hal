use crate::swm::{mode, Pin, PinMode};
use core::marker::PhantomData;

macro_rules! pin {
    ($name:ident, $assign_v:expr) => {
        pub struct $name<'a, MODE: PinMode> {
            regs: &'a PinRegs<'a>,
            _0: PhantomData<MODE>,
        }

        impl<'a, MODE: super::PinMode> Pin for $name<'a, MODE> {
            const NUMBER: u8 = $assign_v;
        }

        impl<'a> $name<'a, mode::Unassigned> {
            pub fn to_digital_output(self, high: bool) -> $name<'a, mode::DigitalOutput> {
                // Set output state first, so that we won't be briefly in the
                // wrong state when we activate the output.
                if high {
                    self.regs
                        .gpio
                        .set0
                        .write(|w| unsafe { w.bits(Self::REG_MASK) });
                } else {
                    self.regs
                        .gpio
                        .clr0
                        .write(|w| unsafe { w.bits(Self::REG_MASK) });
                }
                // Now put the pin in output mode. If the pin was already
                // configured to be a GPIO then we'll start driving the pin
                // after this step.
                // FIXME: This isn't thread safe.
                self.regs
                    .gpio
                    .dir0
                    .modify(|r, w| unsafe { w.bits(r.bits() | Self::REG_MASK) });

                $name {
                    regs: self.regs,
                    _0: PhantomData,
                }
            }
        }

        impl<'a> super::UnassignedPin for $name<'a, mode::Unassigned> {}

        impl<'a, MODE> embedded_hal::digital::v2::InputPin for $name<'a, MODE>
        where
            MODE: PinMode,
        {
            type Error = !;

            fn is_high(&self) -> Result<bool, !> {
                panic!("unimplemented");
            }

            fn is_low(&self) -> Result<bool, !> {
                panic!("unimplemented");
            }
        }

        impl<'a> embedded_hal::digital::v2::OutputPin for $name<'a, mode::DigitalOutput> {
            type Error = !;

            fn set_high(&mut self) -> Result<(), !> {
                panic!("unimplemented");
            }

            fn set_low(&mut self) -> Result<(), !> {
                panic!("unimplemented");
            }
        }
    };
}

pin!(Pin0, 0);
pin!(Pin1, 1);
pin!(Pin2, 2);
pin!(Pin3, 3);
pin!(Pin4, 4);
pin!(Pin5, 5);
pin!(Pin6, 6);
pin!(Pin7, 7);
pin!(Pin8, 8);
pin!(Pin9, 9);
pin!(Pin10, 10);
pin!(Pin11, 11);
pin!(Pin12, 12);
pin!(Pin13, 13);
pin!(Pin14, 14);
pin!(Pin15, 15);
pin!(Pin16, 16);
pin!(Pin17, 17);

macro_rules! pins {
    ( $( $name:ident, $type:ident ),+ ) => {
        pub struct Pins<'a> {
            _regs: &'a PinRegs<'a>,
            $(
                pub $name: $type<'a, super::mode::Unassigned>,
            )*
        }
    };
}

// Note: Pins 2, 3, and 5 are absent here because they are initially owned by
// the SWD and Reset objects. To obtain those, the caller must release them
// from their respective objects in the Parts struct.
pins!(
    gpio0, Pin0, gpio1, Pin1, gpio4, Pin4, gpio6, Pin6, gpio7, Pin7, gpio8, Pin8, gpio9, Pin9,
    gpio10, Pin10, gpio11, Pin11, gpio12, Pin12, gpio13, Pin13, gpio14, Pin14, gpio15, Pin15,
    gpio16, Pin16, gpio17, Pin17
);

struct PinRegs<'a> {
    gpio: &'a lpc81x_pac::GPIO_PORT,
    swm: &'a lpc81x_pac::SWM,
}
