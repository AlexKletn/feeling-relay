use super::{Suid, delivery::Delivery, request_body::RequestBody};

#[derive(prost::Message)]
pub(super) struct Request {
    #[prost(message, required, tag = "1")]
    pub sensor: Suid,
    #[prost(fixed32, required, tag = "2")]
    pub message: u32,
    #[prost(message, required, tag = "3")]
    pub delivery: Delivery,
    #[prost(message, required, tag = "4")]
    pub body: RequestBody,
}

impl Request {
    pub fn wakeup(sensor: Suid, message: u32, payload: Option<Vec<u8>>) -> Self {
        Self {
            sensor,
            message,
            delivery: Delivery {
                wakeup: 1,
                processor: 0,
            },
            body: RequestBody { payload },
        }
    }
}
