#[derive(prost::Message)]
pub(super) struct RequestBody {
    #[prost(bytes, optional, tag = "2")]
    pub payload: Option<Vec<u8>>,
}
