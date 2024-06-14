# Texas Instruments TMP-121 and TMP-123 drivers

This is a platform agnostic Rust driver for Texas Instruments TMP-121 and
TMP-123 SPI devices, using the [`embedded-hal`] (v1) traits.

This driver allows you to:

- Read temperature measurement (see `get_reading()`)

It supports:

- Blocking SPI using `embedded-hal 1.0`

## The devices

An example datasheet can be viewed for the
[TMP-121/TMP-123](https://www.ti.com/lit/ds/symlink/tmp121.pdf)

## Usage

To use this driver, import this crate and an `embedded-hal` (v1) implementation.

The following example shows use of this driver using the
[embassy](https://embassy.dev/) framework:

```rust
let config = embassy_stm32::Config::default();
let p = embassy_stm32::init(config);
let spi_config = embassy_stm32::spi::Config::default();

let cs_pin = embassy_stm32::gpio::Output::new(AnyPin::from(p.PB9), Level::High, Speed::VeryHigh);
let spi = embassy_stm32::spi::Spi::new(
    p.SPI2, p.PF9, p.PB15, p.PA10, p.DMA1_CH5, p.DMA1_CH4, spi_config,
);

// Combine the SPI bus and the CS pin into a SPI device. This now implements SpiDevice!
let spi_device = embedded_hal_bus::spi::ExclusiveDevice::new(spi, cs_pin, embassy_time::Delay).unwrap();
let temp_spi = ti_tmp_12x_rs::comms::Tmp12x::init(spi_device).unwrap();

// Do stuff
let reading = temp_spi.get_reading().unwrap();
```
