use core::fmt::{self, Debug};
use defmt::{Format, Formatter};
use embedded_hal::spi::SpiDevice;

/// The error type used by this library.
///
/// This can encapsulate an SPI or GPIO error, and adds its own protocol errors
/// on top of that.
pub enum Error<SPI: SpiDevice> {
    /// An SPI transfer failed.
    Spi(SPI::Error),

    /// The measurement is invalid (CFM bit is LOW).
    /// Only returned when the `osensa` feature is enabled.
    /// This indicates the FTX 101 sensor has not yet completed a valid measurement.
    #[cfg(feature = "osensa")]
    InvalidMeasurement,

    /// No probe is detected (device returns 0x0000).
    /// Only returned when the `osensa` feature is enabled.
    /// The FTX 101 disables the LED for 10s when no probe is detected to extend unit life.
    #[cfg(feature = "osensa")]
    NoProbe,

    /// Device error detected (device returns 0x7FF8, corresponding to 255°C).
    /// Only returned when the `osensa` feature is enabled.
    /// This may indicate:
    /// - Probe is damaged
    /// - Signal is too weak
    #[cfg(feature = "osensa")]
    DeviceError,
}

impl<SPI: SpiDevice> Format for Error<SPI>
where
    SPI::Error: Debug,
{
    fn format(&self, fmt: Formatter) {
        match self {
            Error::Spi(_spi) => defmt::write!(fmt, "Error::Spi"),
            #[cfg(feature = "osensa")]
            Error::InvalidMeasurement => defmt::write!(fmt, "Error::InvalidMeasurement"),
            #[cfg(feature = "osensa")]
            Error::NoProbe => defmt::write!(fmt, "Error::NoProbe"),
            #[cfg(feature = "osensa")]
            Error::DeviceError => defmt::write!(fmt, "Error::DeviceError"),
        }
    }
}
impl<SPI: SpiDevice> Debug for Error<SPI>
where
    SPI::Error: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Spi(spi) => write!(f, "Error::Spi({:?})", spi),
            #[cfg(feature = "osensa")]
            Error::InvalidMeasurement => write!(f, "Error::InvalidMeasurement"),
            #[cfg(feature = "osensa")]
            Error::NoProbe => write!(f, "Error::NoProbe"),
            #[cfg(feature = "osensa")]
            Error::DeviceError => write!(f, "Error::DeviceError"),
        }
    }
}
