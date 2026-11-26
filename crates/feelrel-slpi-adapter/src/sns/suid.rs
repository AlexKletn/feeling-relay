#[derive(Clone, Copy, PartialEq, Eq, prost::Message)]
pub struct Suid {
    #[prost(fixed64, required, tag = "1")]
    pub low: u64,
    #[prost(fixed64, required, tag = "2")]
    pub high: u64,
}
