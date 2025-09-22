# Texas Instruments TMP-121/TMP-123 and OSENSA FTX 101 drivers

This is a platform agnostic Rust driver for Texas Instruments TMP-121/TMP-123 
and OSENSA FTX 101 OEM SPI temperature sensors, using the [`embedded-hal`] (v1) traits.

This driver allows you to:

- Read temperature measurements (see `get_reading()`)
- Read temperature with LED current diagnostics for FTX 101 (see `get_osensa_reading()` with `osensa` feature)

It supports:

- Blocking SPI using `embedded-hal 1.0`
- Texas Instruments TMP-121 and TMP-123 sensors
- OSENSA FTX 101 OEM sensor (with `osensa` feature flag)

## The devices

### Texas Instruments TMP-121/TMP-123
An example datasheet can be viewed for the
[TMP-121/TMP-123](https://www.ti.com/lit/ds/symlink/tmp121.pdf)

### OSENSA FTX 101 OEM
The FTX 101 OEM sensor is compatible with the TMP123 data format but includes
additional diagnostic features. Reference: Document No: MAN-DMK-0039B, 
Revision No: A, Date: 16-NOV-18.

Key differences when using the `osensa` feature:
- **D2 bit (CFM)**: Confirmation bit indicates measurement validity (HIGH = valid, LOW = invalid)
- **D1, D0 bits**: LED current level indicators
- **Error codes**: 
  - `0x0000`: No probe detected (LED disabled for 10s to extend unit life)
  - `0x7FF8` (255°C): Device error (probe damaged or signal too weak)

## Usage

To use this driver, import this crate and an `embedded-hal` (v1) implementation.

### Basic Usage (TMP-121/TMP-123)

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
let mut temp_spi = ti_tmp_12x_rs::comms::Tmp12x::new(spi_device);

// Do stuff
let reading = temp_spi.get_reading().unwrap();
```

### OSENSA FTX 101 Usage

To use with the OSENSA FTX 101 sensor, enable the `osensa` feature in your `Cargo.toml`:

```toml
[dependencies]
ti-tmp12x-rs = { version = "0.1", features = ["osensa"] }
```

Then use the enhanced API:

```rust
use ti_tmp12x_rs::comms::{Tmp12x, OsensaReading, LedCurrentLevel};

// ... setup SPI as above ...

let mut temp_sensor = Tmp12x::new(spi_device);

// Get temperature with validation (CFM bit checked)
match temp_sensor.get_reading() {
    Ok(temp) => println!("Temperature: {}°C", temp),
    Err(Error::InvalidMeasurement) => println!("Measurement not ready"),
    Err(Error::NoProbe) => println!("No probe detected"),
    Err(Error::DeviceError) => println!("Device error (probe damaged or weak signal)"),
    Err(e) => println!("Other error: {:?}", e),
}

// Get temperature with LED current diagnostics
match temp_sensor.get_osensa_reading() {
    Ok(reading) => {
        println!("Temperature: {}°C", reading.temperature);
        match reading.led_current {
            LedCurrentLevel::Under500 => println!("LED current < 500"),
            LedCurrentLevel::Range500To1000 => println!("LED current 500-1000"),
            LedCurrentLevel::Range1000To2000 => println!("LED current 1000-2000"),
            LedCurrentLevel::Over2000 => println!("LED current > 2000"),
            LedCurrentLevel::Unknown => println!("LED current unknown"),
        }
    },
    Err(e) => println!("Error: {:?}", e),
}
```
