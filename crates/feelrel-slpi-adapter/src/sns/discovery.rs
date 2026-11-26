use prost::Message;

use super::{
    Suid, event_batch::EventBatch, lookup_request::LookupRequest, lookup_result::LookupResult,
    request::Request,
};

const LOOKUP_SUID: Suid = Suid {
    low: 0xabab_abab_abab_abab,
    high: 0xabab_abab_abab_abab,
};
const LOOKUP_REQUEST: u32 = 0x0200;
const LOOKUP_EVENT: u32 = 0x0300;

pub(crate) fn lookup_packet(channel: &str) -> Vec<u8> {
    let query = LookupRequest {
        name: channel.to_owned(),
        register_updates: false,
        default_only: true,
    };
    Request::wakeup(LOOKUP_SUID, LOOKUP_REQUEST, Some(query.encode_to_vec())).encode_to_vec()
}

pub(crate) fn lookup_result(
    payload: &[u8],
    channel: &str,
) -> Result<Option<Suid>, prost::DecodeError> {
    let batch = EventBatch::decode(payload)?;
    if batch.sensor != LOOKUP_SUID {
        return Ok(None);
    }
    for event in batch.events {
        if event.message == LOOKUP_EVENT {
            let result = LookupResult::decode(event.payload.as_slice())?;
            if result.name == channel {
                return Ok(result.sensors.first().copied());
            }
        }
    }
    Ok(None)
}
