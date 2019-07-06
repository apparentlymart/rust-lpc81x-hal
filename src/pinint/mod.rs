//! Interface to the pin interrupt and pattern match engine.

use core::marker::PhantomData;

pub mod int;
pub mod mode;

pub struct Inactive(PhantomData<()>);

impl Inactive {
    pub(crate) fn new() -> Self {
        Self(PhantomData)
    }

    pub fn activate(self) -> PinInterrupts {
        // We assume that the GPIO and pin interrupt clock is already enabled,
        // because that's the reset value and this library offers no way to
        // disable it.

        PinInterrupts {
            int0: int::Interrupt0::new(),
            int1: int::Interrupt1::new(),
            int2: int::Interrupt2::new(),
            int3: int::Interrupt3::new(),
            int4: int::Interrupt4::new(),
            int5: int::Interrupt5::new(),
            int6: int::Interrupt6::new(),
            int7: int::Interrupt7::new(),
        }
    }

    // TODO: Also to_pattern_match_engine, to select the pattern matching
    // mode instead. (The two are mutually-exclusive.)
}

pub struct PinInterrupts {
    pub int0: int::Interrupt0<mode::Inactive>,
    pub int1: int::Interrupt1<mode::Inactive>,
    pub int2: int::Interrupt2<mode::Inactive>,
    pub int3: int::Interrupt3<mode::Inactive>,
    pub int4: int::Interrupt4<mode::Inactive>,
    pub int5: int::Interrupt5<mode::Inactive>,
    pub int6: int::Interrupt6<mode::Inactive>,
    pub int7: int::Interrupt7<mode::Inactive>,
}

pub enum Sensitivity {
    Edge,
    Level,
}
