#[derive(prost::Message)]
pub(super) struct SampleRate {
    #[prost(float, required, tag = "1")]
    pub hz: f32,
}
