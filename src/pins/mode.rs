macro_rules! mode {
    ($name:ident) => {
        pub enum $name {}

        unsafe impl super::PinMode for $name {}
    };
}

mode!(Unassigned);
mode!(DigitalOutput);
mode!(DigitalInput);
mode!(SWM);

unsafe impl super::PinAssignment for Unassigned {}

// A marker type used to represent that a function is assigned to a pin.
pub struct Assigned<PIN: super::Pin>(PIN, void::Void);
unsafe impl<T: super::Pin> super::PinAssignment for Assigned<T> {}
