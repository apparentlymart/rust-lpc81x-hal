use super::{mode, Pin, PinMode};
use core::marker::PhantomData;

macro_rules! pin {
    ($name:ident, $assign_v:expr) => {
        pub struct $name<MODE: PinMode>(PhantomData<MODE>);

        impl<MODE: super::PinMode> $name<MODE> {
            pub(crate) fn new() -> Self {
                Self(PhantomData)
            }

            /// Obtain the input portion of the pin.
            ///
            /// Input functions can coexist on the same pin, so the input
            /// portion can be freely copied and shared even though the other
            /// portions are subject to ownership and move semantics.
            ///
            /// The result of this method implements the embedded-hal digital
            /// v2 `InputPin` trait.
            pub fn digital_input(&self) -> $name<mode::DigitalInput> {
                $name::<mode::DigitalInput>(PhantomData)
            }
        }

        impl<MODE: super::PinMode> !Sync for $name<MODE> {}

        impl $name<mode::Unassigned> {
            /// Configure the pin's output portion for general-purpose digital
            /// output.
            ///
            /// The result of this method implements the embedded-hal digital
            /// v2 `OutputPin` trait.
            ///
            /// If `high` is set then the output will be driving the line high
            /// once activated. Otherwise, it will be driving the line low.
            /// Use the `OutputPin` trait methods to change the pin state after
            /// initial configuration.
            pub fn to_digital_output(self, high: bool) -> $name<mode::DigitalOutput> {
                let gpio = lpc81x_pac::GPIO_PORT::ptr();

                // Set output state first, so that we won't be briefly in the
                // wrong state when we activate the output.
                if high {
                    unsafe { (*gpio).set0.write(|w| w.bits(Self::REG_MASK)) }
                } else {
                    unsafe { (*gpio).clr0.write(|w| w.bits(Self::REG_MASK)) }
                }
                // Now put the pin in output mode. If the pin was already
                // configured to be a GPIO then we'll start driving the pin
                // after this step.
                unsafe {
                    (*gpio)
                        .dir0
                        .modify(|r, w| w.bits(r.bits() | Self::REG_MASK));
                }

                $name(PhantomData)
            }
        }

        unsafe impl<MODE: super::PinMode> Pin for $name<MODE> {
            const NUMBER: u8 = $assign_v;
        }

        unsafe impl super::UnassignedPin for $name<mode::Unassigned> {}
        unsafe impl super::InputPin for $name<mode::DigitalInput> {}
        unsafe impl super::InputPin for $name<mode::Unassigned> {}

        impl embedded_hal::digital::v2::InputPin for $name<mode::DigitalInput> {
            type Error = !;

            fn is_high(&self) -> Result<bool, !> {
                let gpio = lpc81x_pac::GPIO_PORT::ptr();
                Ok(unsafe { (*gpio).b[Self::NUMBER as usize].read().bits() != 0 })
            }

            fn is_low(&self) -> Result<bool, !> {
                let gpio = lpc81x_pac::GPIO_PORT::ptr();
                Ok(unsafe { (*gpio).b[Self::NUMBER as usize].read().bits() == 0 })
            }
        }

        impl embedded_hal::digital::v2::OutputPin for $name<mode::DigitalOutput> {
            type Error = !;

            fn set_high(&mut self) -> Result<(), !> {
                let gpio = lpc81x_pac::GPIO_PORT::ptr();
                unsafe { (*gpio).set0.write(|w| w.bits(Self::REG_MASK)) };
                Ok(())
            }

            fn set_low(&mut self) -> Result<(), !> {
                let gpio = lpc81x_pac::GPIO_PORT::ptr();
                unsafe { (*gpio).clr0.write(|w| w.bits(Self::REG_MASK)) };
                Ok(())
            }
        }

        impl embedded_hal::digital::v2::ToggleableOutputPin for $name<mode::DigitalOutput> {
            type Error = !;

            fn toggle(&mut self) -> Result<(), !> {
                let gpio = lpc81x_pac::GPIO_PORT::ptr();
                unsafe { (*gpio).not0.write(|w| w.bits(Self::REG_MASK)) };
                Ok(())
            }
        }

        /// The input portion of a pin can be freely copied, because multiple
        /// input functions can coexist on the same pin.
        impl core::marker::Copy for $name<mode::DigitalInput> {}
        impl core::clone::Clone for $name<mode::DigitalInput> {
            fn clone(&self) -> Self {
                Self(PhantomData)
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
