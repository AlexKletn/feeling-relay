#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct StreamProfile {
    pub channel: String,
    pub hz: f32,
}
