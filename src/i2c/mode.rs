use core::marker::PhantomData;

pub unsafe trait Status {}

pub struct Active(PhantomData<()>);
pub struct Inactive(PhantomData<()>);

unsafe impl Status for Active {}
unsafe impl Status for Inactive {}

pub struct Host<S: Status>(PhantomData<S>);
pub struct Device<S: Status>(PhantomData<S>);
pub struct Monitor<S: Status>(PhantomData<S>);

pub unsafe trait HostStatus {}
pub unsafe trait DeviceStatus {}
pub unsafe trait MonitorStatus {}

unsafe impl<S: Status> HostStatus for Host<S> {}
unsafe impl<S: Status> DeviceStatus for Device<S> {}
unsafe impl<S: Status> MonitorStatus for Monitor<S> {}

pub type HostActive = Host<Active>;
pub type HostInactive = Host<Inactive>;
pub type DeviceActive = Device<Active>;
pub type DeviceInactive = Device<Inactive>;
pub type MonitorActive = Monitor<Active>;
pub type MonitorInactive = Monitor<Inactive>;
