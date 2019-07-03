pub enum Inactive {}
unsafe impl super::Mode for Inactive {}

pub enum Host {}
unsafe impl super::Mode for Host {}

pub enum Device {}
unsafe impl super::Mode for Device {}

pub unsafe trait Active {}
unsafe impl Active for Host {}
unsafe impl Active for Device {}
