/// Refer to datasheet:
/// https://www.ti.com/lit/ds/symlink/tmp121.pdf
use crate::error::Error;
use core::fmt::Debug;
use embedded_hal::spi::{Operation, SpiDevice};

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

impl<SPI> Tmp12x<SPI>
where
    SPI: SpiDevice,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    /// Get a temperature readings in Celsius
    /// Note - if a value of 0 is returned the sensor is probably not connected
    /// or hasn't read anything yet - so this measurement should probably be
    /// discarded
    pub fn get_reading(&mut self) -> Result<f64, Error<SPI>> {
        let mut words = [0u8; 2];
        self.spi
            .transaction(&mut [Operation::Read(&mut words)])
            .map_err(Error::Spi)?;
        Ok(convert_words(&words))
    }
}

#[cfg(test)]
mod test {
    use super::convert_words;

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
}
