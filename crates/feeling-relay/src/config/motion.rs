use serde::Deserialize;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MotionConfig {
    pub significant: i32,
    pub moving: i32,
    pub stationary: i32,
}
