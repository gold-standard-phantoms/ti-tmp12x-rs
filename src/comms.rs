/// Refer to datasheet:
/// https://www.ti.com/lit/ds/symlink/tmp121.pdf
use crate::error::Error;
use core::fmt::Debug;
use embedded_hal::spi::{Operation, SpiDevice};

/// LED current level indication for OSENSA FTX 101 sensor.
///
/// This enum represents the LED current levels reported by the FTX 101
/// through bits D1 and D0 of the temperature data packet.
#[cfg(feature = "osensa")]
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum LedCurrentLevel {
    /// LED current under 500 (0b00)
    Under500,
    /// LED current is greater than or equal to 500 but less than 1000 (0b01)
    Range500To1000,
    /// LED current is greater than or equal to 1000 but less than 2000 (0b10)
    Range1000To2000,
    /// LED current is greater than or equal to 2000 (0b11)
    Over2000,
    /// Unknown LED current level (should not occur with valid data)
    Unknown,
}

/// Temperature reading with diagnostics from OSENSA FTX 101 sensor.
///
/// This struct contains both the temperature measurement and the
/// LED current level diagnostic information provided by the FTX 101.
#[cfg(feature = "osensa")]
#[derive(Debug, Clone, Copy)]
pub struct OsensaReading {
    /// Temperature measurement in Celsius
    pub temperature: f64,
    /// LED current level indicator
    pub led_current: LedCurrentLevel,
}

pub struct Tmp12x<SPI> {
    spi: SPI,
}
impl<SPI> Debug for Tmp12x<SPI> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "FlashSPI")
    }
}

fn convert_words(words: &[u8; 2]) -> f64 {
    // The Temperature Register of the TMP121 and TMP123 is
    // a 16-bit, signed read-only register that stores the output of
    // the most recent conversion. Up to 16 bits can be read to
    // obtain data and are described in Table 1. The first 13 bits
    // are used to indicate temperature with bits D2 = 0, and D1,
    // D0 in a high impedance state. Data format for temperature
    // is summarized in Table 2. Following power-up or reset, the
    // Temperature Register will read 0°C until the first
    // conversion is complete.
    let all_bits = u16::from_be_bytes(*words);

    // The initial data is has 12 bits plus a bit for signing - all in the
    // MS 13 bits - needs converting to 2's complement
    let is_negative = (all_bits & 0x8000) > 1;
    let sign_mask: u16 = match is_negative {
        true => 0b11110000_00000000u16,
        false => 0b00000000_00000000,
    };
    let temperature = (((all_bits & 0x7FFF) >> 3) | sign_mask) as i16;
    temperature as f64 * 0.0625
}

#[cfg(feature = "osensa")]
fn convert_words_osensa<SPI: embedded_hal::spi::SpiDevice>(words: &[u8; 2]) -> Result<(f64, LedCurrentLevel), Error<SPI>> {
    // For FTX 101: Same temperature format as TMP123 but with additional bits:
    // D2: CFM (confirmation) bit - HIGH = valid, LOW = invalid
    // D1, D0: LED current level indicators
    let all_bits = u16::from_be_bytes(*words);

    // Check for special error values
    if all_bits == 0x0000 {
        // No probe detected
        return Err(Error::NoProbe);
    }
    if all_bits == 0x7FF8 {
        // Device error (255°C)
        return Err(Error::DeviceError);
    }

    // Extract CFM bit (D2) and check validity
    let is_valid = (all_bits & 0x0004) != 0;
    if !is_valid {
        // Measurement is not valid
        return Err(Error::InvalidMeasurement);
    }

    // Extract LED current level (D1, D0)
    let led_bits = (all_bits & 0x0003) as u8;
    let led_current = match led_bits {
        0b00 => LedCurrentLevel::Under500,
        0b01 => LedCurrentLevel::Range500To1000,
        0b10 => LedCurrentLevel::Range1000To2000,
        0b11 => LedCurrentLevel::Over2000,
        _ => LedCurrentLevel::Unknown,
    };

    // Calculate temperature using the existing convert_words function
    let temp_celsius = convert_words(words);

    Ok((temp_celsius, led_current))
}

impl<SPI> Tmp12x<SPI>
where
    SPI: SpiDevice,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Get a temperature reading in Celsius.
    ///
    /// # Behavior
    /// 
    /// When the `osensa` feature is **disabled** (TMP121/TMP123):
    /// - Returns the temperature directly from the sensor
    /// - Note: if 0°C is returned, the sensor might not be connected or hasn't 
    ///   completed its first conversion yet
    ///
    /// When the `osensa` feature is **enabled** (FTX 101):
    /// - Validates the measurement using the CFM (confirmation) bit
    /// - Returns `Err(Error::InvalidMeasurement)` if CFM bit is LOW
    /// - Returns `Err(Error::NoProbe)` if no probe is detected (0x0000)
    /// - Returns `Err(Error::DeviceError)` if device error detected (0x7FF8)
    /// - LED current level is read but discarded (use `get_osensa_reading()` to access it)
    ///
    /// # Errors
    /// 
    /// - `Error::Spi` - SPI communication error
    /// - `Error::InvalidMeasurement` - Measurement not ready (osensa only)
    /// - `Error::NoProbe` - No probe detected (osensa only)
    /// - `Error::DeviceError` - Probe damaged or signal weak (osensa only)
    pub fn get_reading(&mut self) -> Result<f64, Error<SPI>> {
        let mut words = [0u8; 2];
        self.spi
            .transaction(&mut [Operation::Read(&mut words)])
            .map_err(Error::Spi)?;

        #[cfg(feature = "osensa")]
        {
            // For FTX 101, validate the measurement using the CFM bit
            let (temperature, _led_current) = convert_words_osensa::<SPI>(&words)?;
            Ok(temperature)
        }

        #[cfg(not(feature = "osensa"))]
        Ok(convert_words(&words))
    }

    /// Get a temperature reading with LED current diagnostics (OSENSA FTX 101 only).
    ///
    /// This method returns both the temperature measurement and LED current level
    /// information provided by the FTX 101 sensor. The reading is validated using
    /// the CFM (confirmation) bit.
    ///
    /// # Returns
    /// - `Ok(OsensaReading)` - Valid temperature reading with diagnostics
    /// - `Err(Error::InvalidMeasurement)` - CFM bit indicates invalid measurement
    /// - `Err(Error::NoProbe)` - No probe detected (device returns 0x0000)
    /// - `Err(Error::DeviceError)` - Device error (device returns 0x7FF8)
    /// - `Err(Error::Spi)` - SPI communication error
    ///
    /// # Example
    /// ```no_run
    /// # use ti_tmp12x_rs::comms::{Tmp12x, OsensaReading, LedCurrentLevel};
    /// # use embedded_hal::spi::SpiDevice;
    /// # fn example<SPI: SpiDevice>(mut sensor: Tmp12x<SPI>) {
    /// match sensor.get_osensa_reading() {
    ///     Ok(reading) => {
    ///         println!("Temperature: {}°C", reading.temperature);
    ///         match reading.led_current {
    ///             LedCurrentLevel::Under500 => println!("LED current < 500"),
    ///             LedCurrentLevel::Range500To1000 => println!("LED current 500-1000"),
    ///             LedCurrentLevel::Range1000To2000 => println!("LED current 1000-2000"),
    ///             LedCurrentLevel::Over2000 => println!("LED current > 2000"),
    ///             LedCurrentLevel::Unknown => println!("LED current unknown"),
    ///         }
    ///     },
    ///     Err(e) => println!("Error: {:?}", e),
    /// }
    /// # }
    /// ```
    #[cfg(feature = "osensa")]
    pub fn get_osensa_reading(&mut self) -> Result<OsensaReading, Error<SPI>> {
        let mut words = [0u8; 2];
        self.spi
            .transaction(&mut [Operation::Read(&mut words)])
            .map_err(Error::Spi)?;

        let (temperature, led_current) = convert_words_osensa::<SPI>(&words)?;
        Ok(OsensaReading {
            temperature,
            led_current,
        })
    }

}

#[cfg(test)]
mod test {
    use super::convert_words;
    #[cfg(feature = "osensa")]
    use super::{convert_words_osensa, LedCurrentLevel};
    #[cfg(feature = "osensa")]
    use crate::error::Error;

    #[test]
    fn test_word_conversion() {
        // Values from datasheet: https://www.ti.com/lit/ds/symlink/tmp121.pdf
        // table 2
        assert_eq!(convert_words(&[0x4B, 0x00]), 150.0);
        assert_eq!(convert_words(&[0x3E, 0x80]), 125.0);
        assert_eq!(convert_words(&[0x0C, 0x80]), 25.0);
        assert_eq!(convert_words(&[0x00, 0x08]), 0.0625);
        assert_eq!(convert_words(&[0x00, 0x00]), 0.0);
        assert_eq!(convert_words(&[0xFF, 0xF8]), -0.0625);
        assert_eq!(convert_words(&[0xF3, 0x80]), -25.0);
        assert_eq!(convert_words(&[0xE4, 0x80]), -55.0);
    }

    #[cfg(feature = "osensa")]
    #[test]
    fn test_osensa_word_conversion() {
        // Create a dummy SPI type for testing
        use core::convert::Infallible;
        struct DummySpi;
        impl embedded_hal::spi::ErrorType for DummySpi {
            type Error = Infallible;
        }
        impl embedded_hal::spi::SpiDevice for DummySpi {
            fn transaction(&mut self, _operations: &mut [embedded_hal::spi::Operation<'_, u8>]) -> Result<(), Self::Error> {
                Ok(())
            }
        }
        
        // Test valid measurement with CFM bit set (D2 = 1)
        // 25°C with CFM=1, LED current = Under500 (0b00)
        // 0x0C80 = 0000 1100 1000 0000 -> shift right 3 = 0001 1001 = 25°C
        // With CFM bit: 0x0C84 = 0000 1100 1000 0100 (CFM=1, LED=00)
        let result = convert_words_osensa::<DummySpi>(&[0x0C, 0x84]).unwrap();
        assert_eq!(result.0, 25.0); // temperature
        assert_eq!(result.1, LedCurrentLevel::Under500);

        // Test invalid measurement with CFM bit clear (D2 = 0)
        // Should return InvalidMeasurement error
        assert!(matches!(
            convert_words_osensa::<DummySpi>(&[0x0C, 0x80]),
            Err(Error::InvalidMeasurement)
        ));

        // Test LED current levels with valid CFM
        // LED = 0b01 (Range500To1000)
        let result = convert_words_osensa::<DummySpi>(&[0x0C, 0x85]).unwrap();
        assert_eq!(result.1, LedCurrentLevel::Range500To1000);

        // LED = 0b10 (Range1000To2000)
        let result = convert_words_osensa::<DummySpi>(&[0x0C, 0x86]).unwrap();
        assert_eq!(result.1, LedCurrentLevel::Range1000To2000);

        // LED = 0b11 (Over2000)
        let result = convert_words_osensa::<DummySpi>(&[0x0C, 0x87]).unwrap();
        assert_eq!(result.1, LedCurrentLevel::Over2000);

        // Test error conditions
        // No probe detected (0x0000)
        assert!(matches!(
            convert_words_osensa::<DummySpi>(&[0x00, 0x00]),
            Err(Error::NoProbe)
        ));

        // Device error (0x7FF8 = 255°C error code)
        assert!(matches!(
            convert_words_osensa::<DummySpi>(&[0x7F, 0xF8]),
            Err(Error::DeviceError)
        ));

        // Test negative temperature with CFM and LED bits
        // -25°C = 0xF380, with CFM=1, LED=10
        // 0xF386 = 1111 0011 1000 0110
        let result = convert_words_osensa::<DummySpi>(&[0xF3, 0x86]).unwrap();
        assert_eq!(result.0, -25.0);
        assert_eq!(result.1, LedCurrentLevel::Range1000To2000);
    }
}
