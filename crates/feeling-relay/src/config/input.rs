use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct InputConfig {
    pub lid_device: String,
    pub gesture_device: String,
    pub wake_code: i32,
    pub tilt_code: i32,
    pub motion: Option<super::motion::MotionConfig>,
}
