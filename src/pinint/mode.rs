use core::marker::PhantomData;
use crate::pins;

pub trait Sensitivity {}

pub struct Inactive(PhantomData<()>);
impl Sensitivity for Inactive {}

pub trait Sensing: Sensitivity {
    type Pin : pins::Pin;
}

pub struct Edge<PIN: pins::Pin>(PhantomData<PIN>);
impl<PIN: pins::Pin> Sensitivity for Edge<PIN> {}
impl<PIN: pins::Pin> Sensing for Edge<PIN> {
    type Pin = PIN;
}

pub struct Level<PIN: pins::Pin>(PhantomData<PIN>);
impl<PIN: pins::Pin> Sensitivity for Level<PIN> {}
impl<PIN: pins::Pin> Sensing for Level<PIN> {
    type Pin = PIN;
}
