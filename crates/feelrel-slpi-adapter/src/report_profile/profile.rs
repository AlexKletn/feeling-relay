use std::io;

use super::{
    cover::CoverFormat, orientation::OrientationFormat, psmd::PsmdFormat, steps::StepFormat,
    vector::VectorFormat,
};

#[derive(Clone, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ReportProfile {
    pub(crate) acceleration: VectorFormat,
    pub(crate) gyroscope: VectorFormat,
    pub(crate) magnetometer: VectorFormat,
    pub(crate) illuminance: VectorFormat,
    pub(crate) color: VectorFormat,
    pub(crate) gravity: VectorFormat,
    pub(crate) rotation: VectorFormat,
    pub(crate) game_rotation: VectorFormat,
    pub(crate) geomagnetic_rotation: VectorFormat,
    pub(crate) cover: CoverFormat,
    pub(crate) orientation: OrientationFormat,
    pub(crate) steps: StepFormat,
    pub(crate) wake: u32,
    pub(crate) tilt: u32,
    pub(crate) significant_motion: u32,
    pub(crate) psmd: PsmdFormat,
}

impl ReportProfile {
    pub(crate) fn validate(&self) -> io::Result<()> {
        for (name, format, count) in [
            ("acceleration", &self.acceleration, 3),
            ("gyroscope", &self.gyroscope, 3),
            ("magnetometer", &self.magnetometer, 3),
            ("illuminance", &self.illuminance, 1),
            ("color", &self.color, 8),
            ("gravity", &self.gravity, 6),
            ("rotation", &self.rotation, 4),
            ("game_rotation", &self.game_rotation, 4),
            ("geomagnetic_rotation", &self.geomagnetic_rotation, 4),
        ] {
            if format.message == 0
                || !valid_tag(format.field)
                || format.order.len() != count
                || format.scales.as_ref().is_some_and(|scales| {
                    scales.len() != count
                        || scales
                            .iter()
                            .any(|scale| !scale.is_finite() || *scale == 0.0)
                })
            {
                return Err(invalid(name));
            }
        }
        if self.cover.message == 0
            || !valid_tag(self.cover.field)
            || self.cover.open == self.cover.closed
        {
            return Err(invalid("cover"));
        }
        let mut states = self.orientation.states;
        states.sort_unstable();
        if self.orientation.message == 0
            || !valid_tag(self.orientation.field)
            || states.windows(2).any(|pair| pair[0] == pair[1])
        {
            return Err(invalid("orientation"));
        }
        if !valid_tag(self.steps.field)
            || self.steps.messages.is_empty()
            || self.steps.messages.contains(&0)
        {
            return Err(invalid("steps"));
        }
        if [
            self.wake,
            self.tilt,
            self.significant_motion,
            self.psmd.message,
            self.psmd.request,
        ]
        .contains(&0)
            || !valid_tag(self.psmd.mode_field)
            || self.psmd.moving == self.psmd.stationary
        {
            return Err(invalid("motion"));
        }
        Ok(())
    }
}

fn valid_tag(tag: u32) -> bool {
    (1..=0x1fff_ffff).contains(&tag)
}

fn invalid(name: &str) -> io::Error {
    io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("invalid SLPI report profile for {name}"),
    )
}
