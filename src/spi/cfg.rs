pub(crate) const RESET_CONFIG: Config = Config {
    sclk_mode: embedded_hal::spi::MODE_0,
    bit_order: BitOrder::MSBFirst,
};

pub struct Config {
    pub sclk_mode: embedded_hal::spi::Mode,
    pub bit_order: BitOrder,
}

pub enum BitOrder {
    MSBFirst,
    LSBFirst,
}

pub enum Polarity {
    ActiveLow,
    ActiveHigh,
}
