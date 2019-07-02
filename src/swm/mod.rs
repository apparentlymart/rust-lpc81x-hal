pub mod mode;
pub mod pin;

// Trait implemented by types that represent switchable pins.
pub trait Pin {
    // NUMBER is the number that represents the pin in the switch matrix
    // function assignment registers and the bit number to set in the GPIO
    // device registers.
    const NUMBER: u8;

    // REG_MASK is a 32-bit mask for the pin's bit in the GPIO peripheral
    // registers.
    const REG_MASK: u32 = 1 << (Self::NUMBER as u32);
}

// Trait implemented by types representing pins that have not yet been assigned
// a function.
pub trait UnassignedPin {}

// Trait implemented by types representing pin modes.
pub trait PinMode {}
