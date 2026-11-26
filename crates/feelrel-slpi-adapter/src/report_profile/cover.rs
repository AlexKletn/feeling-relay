#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CoverFormat {
    pub message: u32,
    pub field: u32,
    pub open: i32,
    pub closed: i32,
}
