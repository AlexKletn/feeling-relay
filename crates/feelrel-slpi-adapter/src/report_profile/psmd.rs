#[derive(Clone, Copy, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct PsmdFormat {
    pub message: u32,
    pub request: u32,
    pub mode_field: u32,
    pub moving: u32,
    pub stationary: u32,
}
