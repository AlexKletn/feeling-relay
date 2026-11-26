use prost::Message;

use super::{Suid, request::Request, sample_rate::SampleRate};

const ENABLE_STREAM: u32 = 513;
const ENABLE_ON_CHANGE: u32 = 514;

pub(crate) fn stream(sensor: Suid, hz: f32) -> Vec<u8> {
    Request::wakeup(
        sensor,
        ENABLE_STREAM,
        Some(SampleRate { hz }.encode_to_vec()),
    )
    .encode_to_vec()
}

pub(crate) fn on_change(sensor: Suid) -> Vec<u8> {
    Request::wakeup(sensor, ENABLE_ON_CHANGE, None).encode_to_vec()
}

pub(crate) fn psmd(
    sensor: Suid,
    format: crate::report_profile::PsmdFormat,
    moving: bool,
) -> Vec<u8> {
    let mode = if moving {
        format.moving
    } else {
        format.stationary
    };
    let mut payload = Vec::new();
    prost::encoding::uint32::encode(format.mode_field, &mode, &mut payload);
    Request::wakeup(sensor, format.request, Some(payload)).encode_to_vec()
}
