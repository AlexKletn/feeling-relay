use std::io;
use std::num::NonZeroU64;
use std::path::PathBuf;
use std::time::Duration;

use feelrel_slpi_adapter::SlpiProfile;
use serde::Deserialize;

use super::InputConfig;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Config {
    pub slpi: SlpiProfile,
    pub input: InputConfig,
    pub request_timeout_ms: NonZeroU64,
    pub sample_timeout_ms: Option<NonZeroU64>,
    pub iio_accelerometer_device: Option<PathBuf>,
    pub iio_gyroscope_device: Option<PathBuf>,
    pub iio_magnetometer_device: Option<PathBuf>,
    pub iio_measurements_directory: Option<PathBuf>,
}

impl Config {
    pub fn validate(&self) -> io::Result<()> {
        for (device, stream, sensor) in [
            (
                &self.iio_gyroscope_device,
                &self.slpi.gyroscope,
                "gyroscope",
            ),
            (
                &self.iio_magnetometer_device,
                &self.slpi.magnetometer,
                "magnetometer",
            ),
        ] {
            if device.is_some() && stream.is_none() {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    format!("IIO {sensor} requires [slpi.{sensor}]"),
                ));
            }
        }
        Ok(())
    }

    pub fn request_timeout(&self) -> Duration {
        Duration::from_millis(self.request_timeout_ms.get())
    }
    pub fn sample_timeout(&self) -> Duration {
        self.sample_timeout_ms
            .map_or(Duration::from_secs(5), |value| {
                Duration::from_millis(value.get())
            })
    }
}
