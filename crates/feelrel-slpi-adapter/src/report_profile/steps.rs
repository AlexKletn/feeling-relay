#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct StepFormat {
    pub messages: Vec<u32>,
    pub field: u32,
}
