//! I/O pin handling.

pub mod mode;
pub mod pin;

pub(crate) const PINASSIGN_NOTHING: u8 = 0xff;

// Trait implemented by types that represent switchable pins.
//
// Only types in the `lpx81x-hal` crate may implement this trait.
pub unsafe trait Pin {
    // NUMBER is the number that represents the pin in the switch matrix
    // function assignment registers and the bit number to set in the GPIO
    // device registers.
    const NUMBER: u8;

    // REG_MASK is a 32-bit mask for the pin's bit in the GPIO peripheral
    // registers.
    const REG_MASK: u32 = 1 << (Self::NUMBER as u32);
}

// Trait implemented by types representing pins that have not yet been assigned
// an output function.
//
// Only types in the `lpx81x-hal` crate may implement this trait.
pub unsafe trait UnassignedPin: Pin {}

// Trait implemented by types representing the input parts of pins.
//
// Only types in the `lpx81x-hal` crate may implement this trait.
pub unsafe trait InputPin: Pin {}

// Trait implemented by types representing pin modes.
//
// Only types in the `lpx81x-hal` crate may implement this trait.
pub unsafe trait PinMode {}

// Trait implemented by types representing pin assignments.
//
// Only types in the `lpx81x-hal` crate may implement this trait.
pub unsafe trait PinAssignment {}

/// Represents the unassigned pins available for assignment at system reset.
///
/// Move these objects elsewhere to configure the microcontroller's internal
/// switch matrix, routing each pin to an appropriate internal peripheral.
///
/// Note that GPIO pin 3 is not included because at system boot it has been
/// assigned to the serial wire debug (SWD) function SWDIO. To assign another
/// function to that pin, you must deactivate the SWD peripheral (via the
/// main `Parts` object) to obtain the movable pin object for gpio3.
pub struct Pins {
    /// GPIO pin 0. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// The fixed-pin function ACMP_I1 can be activated on this pin only.
    /// Activating ACMP_I2 will cause all digital input functions on this
    /// pin to read consistently low.
    ///
    /// The in-system programming (ISP) mode assigns serial RX function to this
    /// pin when ISP mode is active.
    pub gpio0: pin::Pin0<mode::Unassigned>,

    /// GPIO pin 1. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// The fixed-pin functions ACMP_I2 and CLKIN can be activated on this pin
    /// only. Activating ACMP_I2 will cause all digital input functions on this
    /// pin to read consistently low.
    ///
    /// On an LPC810 (DIP8 package) this pin is sampled on startup by the
    /// bootloader to conditionally enter in-system programming (ISP) mode.
    /// Ensure that any other function assigned to this pin will not cause it
    /// to be pulled low at reset unless entry into ISP mode is desired, or
    /// activate ISP entry protection.
    pub gpio1: pin::Pin1<mode::Unassigned>,

    /// GPIO pin 2. At boot, this pin is in a high-impedance state but is
    /// connected to the SWCLK signal of the serial wire debug interface.
    ///
    /// The SWCLK signal is a fixed-pin function that can be activated on this
    /// pin only.
    ///
    /// Assigning other functions to this pin with the SWCLK function connected
    /// is valid, but the operation of those other functions may conflict with
    /// the operation of the SWD interface. To avoid such conflicts, deactivate
    /// the debug interface.
    pub gpio2: pin::Pin2<mode::Unassigned>,

    /// GPIO pin 4. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// The in-system programming (ISP) mode assigns serial TX function to this
    /// pin when ISP mode is active.
    pub gpio4: pin::Pin4<mode::Unassigned>,

    /// GPIO pin 5. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// The external reset signal can be activated on this pin only.
    pub gpio5: pin::Pin5<mode::Unassigned>,

    /// GPIO pin 6. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// The fixed-pin function VDDCMP can be activated on this pin only.
    /// Activating VDDCMP will cause all digital input functions on this
    /// pin to read consistently low.
    pub gpio6: pin::Pin6<mode::Unassigned>,

    /// GPIO pin 7. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    pub gpio7: pin::Pin7<mode::Unassigned>,

    /// GPIO pin 8. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// The fixed-pin function XTALIN can be activated on this pin only.
    pub gpio8: pin::Pin8<mode::Unassigned>,

    /// GPIO pin 9. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// The fixed-pin function XTALOUT can be activated on this pin only.
    pub gpio9: pin::Pin9<mode::Unassigned>,

    /// GPIO pin 10. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// Although I2C0_SCK can be assigned to any pin, it is open-drain only
    /// when assigned to GPIO pin 10. It can be configured as true open-drain
    /// even if the I2C function is not assigned.
    ///
    /// This pin does not have a programmable pull-up resistor.
    pub gpio10: pin::Pin10<mode::Unassigned>,

    /// GPIO pin 11. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// Although I2C0_SDA can be assigned to any pin, it is open-drain only
    /// when assigned to GPIO pin 11. It can be configured as true open-drain
    /// even if the I2C function is not assigned.
    ///
    /// This pin does not have a programmable pull-up resistor.
    pub gpio11: pin::Pin11<mode::Unassigned>,

    /// GPIO pin 12. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    ///
    /// On LPC811 and LPC812 models this pin is sampled on startup by the
    /// bootloader to conditionally enter in-system programming (ISP) mode.
    /// Ensure that any other function assigned to this pin will not cause it
    /// to be pulled low at reset unless entry into ISP mode is desired, or
    /// activate ISP entry protection.
    pub gpio12: pin::Pin12<mode::Unassigned>,

    /// GPIO pin 13. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    pub gpio13: pin::Pin13<mode::Unassigned>,

    /// GPIO pin 14. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    pub gpio14: pin::Pin14<mode::Unassigned>,

    /// GPIO pin 15. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    pub gpio15: pin::Pin15<mode::Unassigned>,

    /// GPIO pin 16. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    pub gpio16: pin::Pin16<mode::Unassigned>,

    /// GPIO pin 17. At boot, this pin is in a high-impedance state with no
    /// attached input devices.
    pub gpio17: pin::Pin17<mode::Unassigned>,
}

impl Pins {
    pub(crate) fn new() -> Self {
        Self {
            gpio0: pin::Pin0::new(),
            gpio1: pin::Pin1::new(),
            gpio2: pin::Pin2::new(),
            gpio4: pin::Pin4::new(),
            gpio5: pin::Pin5::new(),
            gpio6: pin::Pin6::new(),
            gpio7: pin::Pin7::new(),
            gpio8: pin::Pin8::new(),
            gpio9: pin::Pin9::new(),
            gpio10: pin::Pin10::new(),
            gpio11: pin::Pin11::new(),
            gpio12: pin::Pin12::new(),
            gpio13: pin::Pin13::new(),
            gpio14: pin::Pin14::new(),
            gpio15: pin::Pin15::new(),
            gpio16: pin::Pin16::new(),
            gpio17: pin::Pin17::new(),
        }
    }
}

/// Represents the digital input parts of each of the general-purpose I/O pins.
///
/// Because multiple input functions can coexist on a single pin, the input
/// part of a pin is not subject to "move" behavior can can be freely shared
/// between multiple components. This separate struct exists to ensure that
/// the input parts can always be accessed, even if the output parts (in Pins)
/// have been moved elsewhere.
///
/// Note that although multiple input functions can exist on a single pin, and
/// can optionally coexist with an output function, it's the caller's
/// responsibility to ensure that these functions make sense together. For
/// example, assigning a UART TX and RX to the same pin is valid, but this
/// will effectively turn that pin into a "loopback" interface, causing anything
/// transmitted to be immediately receieved again.
///
/// Refer to the documentation of `Pins` for information about fixed-function
/// pin assignments and how each pin is configured at system boot.
pub struct PinInputs {
    /// GPIO pin 0.
    pub gpio0: pin::Pin0<mode::DigitalInput>,

    /// GPIO pin 1.
    pub gpio1: pin::Pin1<mode::DigitalInput>,

    /// GPIO pin 2.
    pub gpio2: pin::Pin2<mode::DigitalInput>,

    /// GPIO pin 3.
    pub gpio3: pin::Pin3<mode::DigitalInput>,

    /// GPIO pin 4.
    pub gpio4: pin::Pin4<mode::DigitalInput>,

    /// GPIO pin 5. External reset by default.
    ///
    /// This pin is initially configured as an external reset signal, in
    /// which case driving the pin low will cause a system reset regardless
    /// of what other functions might simultaneously be assigned to the pin.
    /// This reset behavior is activated by default at boot, but can be
    /// disabled by deactivating the external reset via the `Parts` struct.
    pub gpio5: pin::Pin5<mode::DigitalInput>,

    /// GPIO pin 6.
    pub gpio6: pin::Pin6<mode::DigitalInput>,

    /// GPIO pin 7.
    pub gpio7: pin::Pin7<mode::DigitalInput>,

    /// GPIO pin 8.
    pub gpio8: pin::Pin8<mode::DigitalInput>,

    /// GPIO pin 9.
    pub gpio9: pin::Pin9<mode::DigitalInput>,

    /// GPIO pin 10.
    pub gpio10: pin::Pin10<mode::DigitalInput>,

    /// GPIO pin 11.
    pub gpio11: pin::Pin11<mode::DigitalInput>,

    /// GPIO pin 12.
    pub gpio12: pin::Pin12<mode::DigitalInput>,

    /// GPIO pin 13.
    pub gpio13: pin::Pin13<mode::DigitalInput>,

    /// GPIO pin 14.
    pub gpio14: pin::Pin14<mode::DigitalInput>,

    /// GPIO pin 15.
    pub gpio15: pin::Pin15<mode::DigitalInput>,

    /// GPIO pin 16.
    pub gpio16: pin::Pin16<mode::DigitalInput>,

    /// GPIO pin 17.
    pub gpio17: pin::Pin17<mode::DigitalInput>,
}

impl PinInputs {
    pub(crate) fn new() -> Self {
        Self {
            gpio0: pin::Pin0::new(),
            gpio1: pin::Pin1::new(),
            gpio2: pin::Pin2::new(),
            gpio3: pin::Pin3::new(),
            gpio4: pin::Pin4::new(),
            gpio5: pin::Pin5::new(),
            gpio6: pin::Pin6::new(),
            gpio7: pin::Pin7::new(),
            gpio8: pin::Pin8::new(),
            gpio9: pin::Pin9::new(),
            gpio10: pin::Pin10::new(),
            gpio11: pin::Pin11::new(),
            gpio12: pin::Pin12::new(),
            gpio13: pin::Pin13::new(),
            gpio14: pin::Pin14::new(),
            gpio15: pin::Pin15::new(),
            gpio16: pin::Pin16::new(),
            gpio17: pin::Pin17::new(),
        }
    }
}
