#[derive(prost::Message)]
pub(super) struct LookupRequest {
    #[prost(string, required, tag = "1")]
    pub name: String,
    #[prost(bool, required, tag = "2")]
    pub register_updates: bool,
    #[prost(bool, required, tag = "3")]
    pub default_only: bool,
}
