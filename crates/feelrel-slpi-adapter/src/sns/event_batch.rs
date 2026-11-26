use super::{Suid, event::Event};

#[derive(prost::Message)]
pub(crate) struct EventBatch {
    #[prost(message, required, tag = "1")]
    pub sensor: Suid,
    #[prost(message, repeated, tag = "2")]
    pub events: Vec<Event>,
}
