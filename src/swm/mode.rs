macro_rules! mode {
    ($name:ident) => {
        pub enum $name {}

        impl super::PinMode for $name {}
    };
}

mode!(Unassigned);
mode!(DigitalOutput);
