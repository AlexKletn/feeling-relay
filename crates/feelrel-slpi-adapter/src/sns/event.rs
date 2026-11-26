#[derive(prost::Message)]
pub(crate) struct Event {
    #[prost(fixed32, required, tag = "1")]
    pub message: u32,
    #[prost(fixed64, required, tag = "2")]
    pub timestamp: u64,
    #[prost(bytes, tag = "3")]
    pub payload: Vec<u8>,
}
