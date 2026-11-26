#[derive(prost::Message)]
pub(super) struct Delivery {
    #[prost(uint32, required, tag = "1")]
    pub wakeup: u32,
    #[prost(uint32, required, tag = "2")]
    pub processor: u32,
}
