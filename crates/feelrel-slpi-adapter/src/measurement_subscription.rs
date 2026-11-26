use crate::{
    StreamProfile, Suid,
    measurement_kind::MeasurementKind,
    sns::{on_change, stream},
};

pub(super) struct MeasurementSubscription {
    pub sensor: Suid,
    pub kind: MeasurementKind,
    pub hz: Option<f32>,
}

impl MeasurementSubscription {
    pub fn packet(&self) -> Vec<u8> {
        self.hz
            .map_or_else(|| on_change(self.sensor), |hz| stream(self.sensor, hz))
    }
}

pub(super) fn descriptor(profile: Option<&StreamProfile>) -> Option<(&str, Option<f32>)> {
    profile.map(|stream| (stream.channel.as_str(), Some(stream.hz)))
}
