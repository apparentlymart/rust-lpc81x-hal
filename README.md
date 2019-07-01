# LPC81x HAL

## Introduction

Hardware Abstraction Layer (HAL) for [NXP LPC81x] microcontrollers, written in the [Rust] programming language.

[nxp lpc82x]: https://www.nxp.com/products/processors-and-microcontrollers/arm-based-processors-and-mcus/general-purpose-mcus/lpc800-cortex-m0-plus-/low-cost-microcontrollers-mcus-based-on-arm-cortex-m0-plus-cores:LPC81X_LPC83X
[rust]: https://www.rust-lang.org/
[embedded-hal]: https://crates.io/crates/embedded-hal

## Status

LPC81x HAL is still under heavy development. It is lacking APIs for many peripherals, and the APIs that already exist are mostly incomplete.

## License

This project is open source software, licensed under the terms of the [Zero Clause BSD License][]. See [LICENSE] for full details.

[zero clause bsd license]: https://opensource.org/licenses/FPL-1.0.0
[license]: ./LICENSE

This library is a fork of [lpc82x-hal], Copyright (c) 2017 Hanno Braun.

[lpc82x-hal]: https://crates.io/crates/lpc82x-hal
