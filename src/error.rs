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
    /// Only returned when decoding as an OSENSA FTX 101 ([`crate::comms::SensorKind::OsensaFtx101`]).
    /// This indicates the FTX 101 sensor has not yet completed a valid measurement.
    InvalidMeasurement,

    /// No probe is detected (device returns 0x0000).
    /// Only returned when decoding as an OSENSA FTX 101 ([`crate::comms::SensorKind::OsensaFtx101`]).
    /// The FTX 101 disables the LED for 10s when no probe is detected to extend unit life.
    NoProbe,

    /// Device error detected (device returns 0x7FF8, corresponding to 255°C).
    /// Only returned when decoding as an OSENSA FTX 101 ([`crate::comms::SensorKind::OsensaFtx101`]).
    /// This may indicate:
    /// - Probe is damaged
    /// - Signal is too weak
    DeviceError,
}

impl<SPI: SpiDevice> Format for Error<SPI>
where
    SPI::Error: Debug,
{
    fn format(&self, fmt: Formatter) {
        match self {
            Error::Spi(_spi) => defmt::write!(fmt, "Error::Spi"),
                    Error::InvalidMeasurement => defmt::write!(fmt, "Error::InvalidMeasurement"),
                    Error::NoProbe => defmt::write!(fmt, "Error::NoProbe"),
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
                    Error::InvalidMeasurement => write!(f, "Error::InvalidMeasurement"),
                    Error::NoProbe => write!(f, "Error::NoProbe"),
                    Error::DeviceError => write!(f, "Error::DeviceError"),
        }
    }
}
