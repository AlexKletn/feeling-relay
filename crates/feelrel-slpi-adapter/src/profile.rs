use std::num::NonZeroU32;

use crate::StreamProfile;

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SlpiProfile {
    pub reports: crate::ReportProfile,
    #[serde(default)]
    pub measurements: crate::MeasurementProfile,
    pub accelerometer_channel: String,
    pub cover_channel: String,
    pub wake_gesture_channel: String,
    pub tilt_channel: String,
    pub accelerometer_hz: f32,
    pub gyroscope: Option<StreamProfile>,
    pub magnetometer: Option<StreamProfile>,
    pub timestamp_hz: NonZeroU32,
}
