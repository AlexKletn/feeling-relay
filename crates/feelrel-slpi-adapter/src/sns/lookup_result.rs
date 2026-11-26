use super::Suid;

#[derive(prost::Message)]
pub(super) struct LookupResult {
    #[prost(string, required, tag = "1")]
    pub name: String,
    #[prost(message, repeated, tag = "2")]
    pub sensors: Vec<Suid>,
}
