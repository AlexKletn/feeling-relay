use std::io;
use std::time::Duration;

use feelrel_ports::{
    Acceleration, ColorSample, GravitySample, Illuminance, Motion, Orientation, Rotation,
    RotationReference, StepCount, Timestamped,
};

use crate::{
    MeasurementProfile, ReportProfile, SlpiConnection, Suid,
    event_queue::EventQueue,
    measurement_kind::MeasurementKind,
    measurement_subscription::{MeasurementSubscription, descriptor},
    sns::{EventBatch, integer},
};

#[derive(Default)]
pub(super) struct Measurements {
    subscriptions: Vec<MeasurementSubscription>,
    pub light: EventQueue<Timestamped<Illuminance>>,
    pub color: EventQueue<Timestamped<ColorSample>>,
    pub gravity: EventQueue<Timestamped<GravitySample>>,
    pub rotation: EventQueue<Timestamped<Rotation>>,
    pub game_rotation: EventQueue<Timestamped<Rotation>>,
    pub geomagnetic_rotation: EventQueue<Timestamped<Rotation>>,
    pub orientation: EventQueue<Timestamped<Orientation>>,
    pub steps: EventQueue<Timestamped<StepCount>>,
    pub motion: EventQueue<Timestamped<Motion>>,
    pub pending_rearm: Option<Suid>,
}

impl Measurements {
    pub async fn discover(
        connection: &mut SlpiConnection,
        profile: &MeasurementProfile,
        wait: Duration,
    ) -> io::Result<Self> {
        let channels = [
            (
                MeasurementKind::Illuminance,
                profile.illuminance.as_deref().map(on_change),
            ),
            (MeasurementKind::Color, descriptor(profile.color.as_ref())),
            (
                MeasurementKind::Gravity,
                descriptor(profile.gravity.as_ref()),
            ),
            (
                MeasurementKind::Rotation(RotationReference::Fused),
                descriptor(profile.rotation.as_ref()),
            ),
            (
                MeasurementKind::Rotation(RotationReference::Game),
                descriptor(profile.game_rotation.as_ref()),
            ),
            (
                MeasurementKind::Rotation(RotationReference::Geomagnetic),
                descriptor(profile.geomagnetic_rotation.as_ref()),
            ),
            (
                MeasurementKind::Orientation,
                profile.orientation.as_deref().map(on_change),
            ),
            (
                MeasurementKind::Pedometer,
                profile.pedometer.as_deref().map(on_change),
            ),
            (
                MeasurementKind::SignificantMotion,
                profile.significant_motion.as_deref().map(on_change),
            ),
        ];
        if channels.iter().any(|(_, channel)| {
            channel.is_some_and(|(_, hz)| hz.is_some_and(|hz| !hz.is_finite() || hz <= 0.0))
        }) {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid SLPI measurement rate",
            ));
        }
        let mut result = Self::default();
        for (kind, channel) in channels {
            if let Some((name, hz)) = channel {
                let sensor = connection.discover_channel(name, wait).await?;
                if result
                    .subscriptions
                    .iter()
                    .any(|existing| existing.sensor == sensor)
                {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "duplicate SLPI measurement channel",
                    ));
                }
                result
                    .subscriptions
                    .push(MeasurementSubscription { sensor, kind, hz });
            }
        }
        Ok(result)
    }

    pub async fn subscribe(&self, connection: &mut SlpiConnection) -> io::Result<()> {
        for subscription in &self.subscriptions {
            connection.send_request(&subscription.packet()).await?;
        }
        Ok(())
    }

    pub fn accept(
        &mut self,
        batch: &EventBatch,
        reports: &ReportProfile,
        timestamp: impl Fn(u64) -> Duration,
    ) -> io::Result<bool> {
        let Some(kind) = self
            .subscriptions
            .iter()
            .find(|sub| sub.sensor == batch.sensor)
            .map(|sub| sub.kind)
        else {
            return Ok(false);
        };
        for event in &batch.events {
            let time = timestamp(event.timestamp);
            let payload = event.payload.as_slice();
            let format = match kind {
                MeasurementKind::Illuminance => Some(&reports.illuminance),
                MeasurementKind::Color => Some(&reports.color),
                MeasurementKind::Gravity => Some(&reports.gravity),
                MeasurementKind::Rotation(RotationReference::Fused) => Some(&reports.rotation),
                MeasurementKind::Rotation(RotationReference::Game) => Some(&reports.game_rotation),
                MeasurementKind::Rotation(RotationReference::Geomagnetic) => {
                    Some(&reports.geomagnetic_rotation)
                }
                _ => None,
            };
            if let Some(format) = format {
                if event.message == format.message
                    && let Some(values) = format.select(payload)?
                {
                    self.accept_values(kind, time, &values);
                }
                continue;
            }
            match kind {
                MeasurementKind::Orientation if event.message == reports.orientation.message => {
                    let code =
                        integer(payload, reports.orientation.field)?.map(|value| value as i32);
                    let value = code
                        .and_then(|code| {
                            reports
                                .orientation
                                .states
                                .iter()
                                .position(|state| *state == code)
                        })
                        .map_or(Orientation::Unknown, |index| {
                            [
                                Orientation::Upright,
                                Orientation::Counterclockwise,
                                Orientation::UpsideDown,
                                Orientation::Clockwise,
                            ][index]
                        });
                    self.orientation.push(Timestamped {
                        timestamp: time,
                        value,
                    });
                }
                MeasurementKind::Pedometer if reports.steps.messages.contains(&event.message) => {
                    if let Some(steps) = integer(payload, reports.steps.field)?
                        .and_then(|value| u32::try_from(value).ok())
                    {
                        self.steps.push(Timestamped {
                            timestamp: time,
                            value: StepCount { steps },
                        });
                    }
                }
                MeasurementKind::SignificantMotion
                    if event.message == reports.significant_motion =>
                {
                    self.motion.push(Timestamped {
                        timestamp: time,
                        value: Motion::Significant,
                    });
                    self.pending_rearm = Some(batch.sensor);
                }
                _ => {}
            }
        }
        Ok(true)
    }

    pub fn rotation_queue(
        &mut self,
        reference: RotationReference,
    ) -> &mut EventQueue<Timestamped<Rotation>> {
        match reference {
            RotationReference::Fused => &mut self.rotation,
            RotationReference::Game => &mut self.game_rotation,
            RotationReference::Geomagnetic => &mut self.geomagnetic_rotation,
        }
    }

    fn accept_values(&mut self, kind: MeasurementKind, timestamp: Duration, axes: &[f32]) {
        match (kind, axes) {
            (MeasurementKind::Illuminance, &[lux, ..]) => {
                self.light.push(Timestamped {
                    timestamp,
                    value: Illuminance { lux },
                });
            }
            (
                MeasurementKind::Color,
                &[
                    temperature_kelvin,
                    clear,
                    red,
                    green,
                    blue,
                    white,
                    integration_ms,
                    gain,
                    ..,
                ],
            ) => {
                self.color.push(Timestamped {
                    timestamp,
                    value: ColorSample {
                        temperature_kelvin,
                        clear,
                        red,
                        green,
                        blue,
                        white,
                        integration_ms,
                        gain,
                    },
                });
            }
            (MeasurementKind::Gravity, &[gx, gy, gz, lx, ly, lz, ..]) => {
                self.gravity.push(Timestamped {
                    timestamp,
                    value: GravitySample {
                        gravity: Acceleration {
                            x_mps2: gx,
                            y_mps2: gy,
                            z_mps2: gz,
                        },
                        linear_acceleration: Acceleration {
                            x_mps2: lx,
                            y_mps2: ly,
                            z_mps2: lz,
                        },
                    },
                });
            }
            (MeasurementKind::Rotation(reference), &[x, y, z, w, ..]) => {
                self.rotation_queue(reference).push(Timestamped {
                    timestamp,
                    value: Rotation { x, y, z, w },
                });
            }
            _ => {}
        }
    }
}

fn on_change(channel: &str) -> (&str, Option<f32>) {
    (channel, None)
}

#[cfg(test)]
#[path = "measurements.tests.rs"]
mod tests;
