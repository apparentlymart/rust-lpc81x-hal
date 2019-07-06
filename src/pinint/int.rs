use super::mode;
use crate::pins;
use core::marker::PhantomData;

macro_rules! pinint {
    ($name:ident, $idx:expr) => {
        pub struct $name<MODE: mode::Sensitivity>(PhantomData<MODE>);

        impl<MODE: mode::Sensitivity> $name<MODE> {
            pub fn new() -> Self {
                Self(PhantomData)
            }
        }

        impl $name<mode::Inactive> {
            /// Consumes the inactive interrupt pin and returns it configured
            /// to detect edges on the given pin.
            ///
            /// The interrupt is not automatically enabled. Call `enable` on
            /// the result to actually begin receiving interrupts.
            pub fn edge_triggered<P: pins::InputPin>(self, pin: P) -> $name<mode::Edge<P>> {
                let syscon = lpc81x_pac::SYSCON::ptr();
                let periph = lpc81x_pac::PIN_INT::ptr();
                unsafe {
                    (*syscon).pintsel[$idx].write(|w| w.intpin().bits(P::NUMBER));
                    (*periph)
                        .isel
                        .modify(|r, w| w.pmode().bits(r.pmode().bits() & !(1 << $idx)));
                }
                unused(pin);
                $name(PhantomData)
            }

            /// Consumes the inactive interrupt pin and returns it configured
            /// to detect levels on the given pin.
            ///
            /// The interrupt is not automatically enabled. Call `enable` on
            /// the result to actually begin receiving interrupts.
            pub fn level_triggered<P: pins::InputPin>(self, pin: P) -> $name<mode::Level<P>> {
                let syscon = lpc81x_pac::SYSCON::ptr();
                let periph = lpc81x_pac::PIN_INT::ptr();
                unsafe {
                    (*syscon).pintsel[$idx].write(|w| w.intpin().bits(P::NUMBER));
                    (*periph)
                        .isel
                        .modify(|r, w| w.pmode().bits(r.pmode().bits() | 1 << $idx));
                }
                unused(pin);
                $name(PhantomData)
            }
        }

        impl<MODE, PIN> $name<MODE>
        where
            MODE: mode::Sensing<Pin = PIN>,
            PIN: pins::Pin,
        {
            pub const NVIC_BITMASK: u32 = 1 << (24 + ($idx));

            /// Returns the pin that is currently connected to the pin interrupt.
            #[inline(always)]
            pub fn pin(&self) -> PIN {
                pin_type_as_is()
            }

            /// Enable this interrupt.
            #[inline(always)]
            pub fn enable(&self, rising: bool, falling: bool) {
                let nvic = lpc81x_pac::NVIC::ptr();
                let periph = lpc81x_pac::PIN_INT::ptr();
                unsafe {
                    (*nvic).iser[0].write(Self::NVIC_BITMASK);
                }
                if rising {
                    unsafe {
                        (*periph).sienr.write(|w| w.setenrl().bits(1 << $idx));
                    }
                }
                if falling {
                    unsafe {
                        (*periph).sienf.write(|w| w.setenaf().bits(1 << $idx));
                    }
                }
            }

            /// Disable this interrupt.
            #[inline(always)]
            pub fn disable(&self) {
                let nvic = lpc81x_pac::NVIC::ptr();
                let periph = lpc81x_pac::PIN_INT::ptr();
                unsafe {
                    (*periph).cienr.write(|w| w.cenrl().bits(1 << $idx));
                    (*periph).cienf.write(|w| w.cenaf().bits(1 << $idx));
                    (*nvic).icer[0].write(Self::NVIC_BITMASK);
                }
            }

            /// Clear any active rising or falling edge notifications.
            ///
            /// The interrupt service routine must call this before returning
            /// or else it will be immediately called again as soon as it
            /// returns.
            #[inline(always)]
            pub fn acknowledge_events(&self) {
                let periph = lpc81x_pac::PIN_INT::ptr();
                unsafe {
                    (*periph).ist.write(|w| {
                        w.pstat().bits(1 << $idx) // Clear pinint1
                    });
                }
            }

            /// Consumes the pin interrupt and returns it deactivated, along
            /// with the pin it was previously monitoring.
            pub fn release_pin(self) -> ($name<mode::Inactive>, PIN) {
                let syscon = lpc81x_pac::SYSCON::ptr();
                let periph = lpc81x_pac::PIN_INT::ptr();
                self.disable();
                unsafe {
                    (*syscon).pintsel[$idx].write(|w| {
                        // The reset value is zero, but that's also how we'd
                        // select pin zero so it's important to call
                        // self.disable before we set this to avoid spurious
                        // interrupts.
                        w.intpin().bits(0)
                    });
                    (*periph)
                        .isel
                        .modify(|r, w| w.pmode().bits(r.pmode().bits() & !(1 << $idx)));
                }

                ($name(PhantomData), pin_type_as_is())
            }
        }
    };
}

pinint!(Interrupt0, 0);
pinint!(Interrupt1, 1);
pinint!(Interrupt2, 2);
pinint!(Interrupt3, 3);
pinint!(Interrupt4, 4);
pinint!(Interrupt5, 5);
pinint!(Interrupt6, 6);
pinint!(Interrupt7, 7);

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
