#![cfg(target_os = "linux")]

mod device_name;
mod gesture_device;
mod iio_accelerometer;
mod iio_gyroscope;
mod iio_magnetometer;
mod iio_measurements;
mod iio_vector;
mod lid_device;
mod motion_codes;

pub use gesture_device::GestureDevice;
pub use iio_accelerometer::IioAccelerometer;
pub use iio_gyroscope::IioGyroscope;
pub use iio_magnetometer::IioMagnetometer;
pub use iio_measurements::IioMeasurements;
pub use lid_device::LidDevice;
pub use motion_codes::MotionCodes;
