//! API for the Power Management Unit (PMU)
//!
//! The PMU is described in the user manual, chapter 6.
//!
//! # Examples
//!
//! Use the PMU to enter sleep mode:
//!
//! ``` no_run
//! extern crate lpc82x;
//! extern crate lpc82x_hal;
//!
//! use lpc82x_hal::Peripherals;
//!
//! let mut core_peripherals = lpc82x::CorePeripherals::take().unwrap();
//! let mut peripherals      = Peripherals::take().unwrap();
//!
//! let mut pmu = peripherals.pmu.split();
//!
//! // Enters sleep mode. Unless we set up some interrupts, we won't wake up
//! // from this again.
//! pmu.handle.enter_sleep_mode(&mut core_peripherals.SCB);
//! ```
//!
//! [`PMU`]: struct.PMU.html
//! [`Peripherals`]: ../struct.Peripherals.html
//! [`pmu::Handle`]: struct.Handle.html
//! [`lpc82x::PMU`]: https://docs.rs/lpc82x/0.3.*/lpc82x/struct.PMU.html


use cortex_m::{
    asm,
    interrupt,
};

use clock;
use init_state::{
    self,
    InitState,
};
use raw;


/// Entry point to the PMU API
pub struct PMU {
    pmu: raw::PMU,
}

impl PMU {
    pub(crate) fn new(pmu: raw::PMU) -> Self {
        PMU { pmu }
    }

    /// Splits the PMU API into its parts
    pub fn split(self) -> Parts {
        Parts {
            handle: Handle {
                pmu: self.pmu,
            },
            low_power_clock: LowPowerClock::new(),
        }
    }

    /// Return the raw peripheral
    pub fn free(self) -> raw::PMU {
        self.pmu
    }
}


/// The main API for the PMU peripheral
///
/// Provides access to all types that make up the PMU API. Please refer to the
/// [module documentation] for more information.
///
/// [module documentation]: index.html
pub struct Parts {
    /// The handle to the PMU peripheral
    pub handle: Handle,

    /// The 10 kHz low-power clock
    pub low_power_clock: LowPowerClock<init_state::Disabled>,
}


/// The handle to the PMU peripheral
///
/// Please refer to the [module documentation] for more information about the
/// PMU.
///
/// [module documentation]: index.html
pub struct Handle {
    pmu: raw::PMU,
}

impl Handle {
    /// Enter sleep mode
    ///
    /// The microcontroller will wake up from sleep mode, if an NVIC-enabled
    /// interrupt occurs. See user manual, section 6.7.4.3.
    pub fn enter_sleep_mode(&mut self, scb: &mut raw::SCB) {
        interrupt::free(|_| {
            // Default power mode indicates active or sleep mode.
            self.pmu.pcon.modify(|_, w|
                w.pm().default()
            );

            // The SLEEPDEEP bit must be cleared when entering regular sleep
            // mode. See user manual, section 6.7.4.2.
            scb.clear_sleepdeep();

            asm::dsb();
            asm::wfi();
        })
    }

    /// Enter deep-sleep mode
    ///
    /// The microcontroller will wake up from deep-sleep mode, if an
    /// NVIC-enabled interrupt occurs. See user manual, section 6.7.5.3.
    ///
    /// # Limitations
    ///
    /// According to the user manual, section 6.7.5.2, the IRC must be selected
    /// as the main clock before entering deep-sleep mode.
    ///
    /// If you intend to wake up from this mode again, you need to configure the
    /// STARTERP0 and STARTERP1 registers of the SYSCON appropriately. See user
    /// manual, section 6.5.1.
    ///
    /// # Safety
    ///
    /// The configuration of various peripherals after wake-up is controlled by
    /// the PDAWAKECFG register. If the configuration in that register doesn't
    /// match the peripheral states in this API, you can confuse the API into
    /// believing that peripherals have a different state than they actually
    /// have which can lead to all kinds of adverse consequences.
    ///
    /// Please make sure that the peripheral states configured in PDAWAKECFG
    /// match the peripheral states as tracked by the API before calling this
    /// method.
    pub unsafe fn enter_deep_sleep_mode(&mut self, scb: &mut raw::SCB) {
        interrupt::free(|_| {
            self.pmu.pcon.modify(|_, w|
                w.pm().deep_sleep_mode()
            );

            // The SLEEPDEEP bit must be set for entering regular sleep mode.
            // See user manual, section 6.7.5.2.
            scb.set_sleepdeep();

            asm::dsb();
            asm::wfi();
        })
    }

    /// Enter power-down mode
    ///
    /// The microcontroller will wake up from power-down mode, if an
    /// NVIC-enabled interrupt occurs. See user manual, section 6.7.6.3.
    ///
    /// # Limitations
    ///
    /// According to the user manual, section 6.7.6.2, the IRC must be selected
    /// as the main clock before entering deep-sleep mode.
    ///
    /// If you intend to wake up from this mode again, you need to configure the
    /// STARTERP0 and STARTERP1 registers of the SYSCON appropriately. See user
    /// manual, section 6.5.1.
    ///
    /// # Safety
    ///
    /// The configuration of various peripherals after wake-up is controlled by
    /// the PDAWAKECFG register. If the configuration in that register doesn't
    /// match the peripheral states in this API, you can confuse the API into
    /// believing that peripherals have a different state than they actually
    /// have which can lead to all kinds of adverse consequences.
    ///
    /// Please make sure that the peripheral states configured in PDAWAKECFG
    /// match the peripheral states as tracked by the API before calling this
    /// method.
    pub unsafe fn enter_power_down_mode(&mut self, scb: &mut raw::SCB) {
        interrupt::free(|_| {
            self.pmu.pcon.modify(|_, w|
                w.pm().power_down_mode()
            );

            // The SLEEPDEEP bit must be set for entering regular sleep mode.
            // See user manual, section 6.7.5.2.
            scb.set_sleepdeep();

            asm::dsb();
            asm::wfi();
        })
    }
}


/// The 10 kHz low-power clock
///
/// This is one of the clocks that can be used to run the self-wake-up timer
/// (WKT). See user manual, section 18.5.1.
pub struct LowPowerClock<State: InitState = init_state::Enabled> {
    _state: State,
}

impl LowPowerClock<init_state::Disabled> {
    pub(crate) fn new() -> Self {
        LowPowerClock {
            _state: init_state::Disabled,
        }
    }
}

impl LowPowerClock<init_state::Disabled> {
    /// Enable the low-power clock
    ///
    /// This method is only available if the low-power clock is not already
    /// enabled. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// Consumes this instance of `LowPowerClock` and returns a new instance
    /// whose state indicates that the clock is enabled. That new instance
    /// implements [`clock::Enabled`], which might be required by APIs that need
    /// an enabled clock.
    ///
    /// [`clock::Enabled`]: ../clock/trait.Enabled.html
    pub fn enable(self, pmu: &mut Handle)
        -> LowPowerClock<init_state::Enabled>
    {
        pmu.pmu.dpdctrl.modify(|_, w|
            w.lposcen().enabled()
        );

        LowPowerClock {
            _state: init_state::Enabled,
        }
    }
}

impl LowPowerClock<init_state::Enabled> {
    /// Disable the low-power clock
    ///
    /// This method is only available if the low-power clock is not already
    /// disabled. Code attempting to call this method when this is not the case
    /// will not compile.
    ///
    /// Consumes this instance of `LowPowerClock` and returns a new instance
    /// whose state indicates that the clock is disabled.
    pub fn disable(self, pmu: &mut Handle)
        -> LowPowerClock<init_state::Disabled>
    {
        pmu.pmu.dpdctrl.modify(|_, w|
            w.lposcen().disabled()
        );

        LowPowerClock {
            _state: init_state::Disabled,
        }
    }
}

impl<State> clock::Frequency for LowPowerClock<State> where State: InitState {
    fn hz(&self) -> u32 { 10_000 }
}

impl clock::Enabled for LowPowerClock<init_state::Enabled> {}
