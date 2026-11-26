#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct OrientationFormat {
    pub message: u32,
    pub field: u32,
    pub states: [i32; 4],
}
