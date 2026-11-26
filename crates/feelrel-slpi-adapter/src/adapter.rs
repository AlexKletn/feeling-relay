use std::io;
use std::num::NonZeroU32;
use std::time::Duration;

use feelrel_ports::{
    Acceleration, AccelerometerPort, AngularVelocity, CoverPort, CoverState, Gesture, GesturePort,
    GyroscopePort, MagneticField, MagnetometerPort, Timestamped,
};
use feelrel_ports::{
    ColorPort, ColorSample, GravityPort, GravitySample, Illuminance, IlluminancePort, Motion,
    MotionPort, Orientation, OrientationPort, PedometerPort, Rotation, RotationPort,
    RotationReference, StepCount,
};
use tokio::time::timeout;

use crate::connection::MAX_QMI_DATAGRAM;
use crate::event_queue::EventQueue;
use crate::measurements::Measurements;
use crate::motion_subscription::{MotionSubscription, next_motion};
use crate::sensors::Sensors;
use crate::sns::{EventBatch, integer, on_change, stream};
use crate::{ReportProfile, SlpiConnection, SlpiProfile};

pub struct SlpiAdapter {
    connection: SlpiConnection,
    reports: ReportProfile,
    sensors: Sensors,
    measurements: Measurements,
    moving: Option<MotionSubscription>,
    stationary: Option<MotionSubscription>,
    timestamp_hz: NonZeroU32,
    buffer: Vec<u8>,
    acceleration: EventQueue<Timestamped<Acceleration>>,
    angular_velocity: EventQueue<Timestamped<AngularVelocity>>,
    magnetic_field: EventQueue<Timestamped<MagneticField>>,
    cover: Option<Timestamped<CoverState>>,
    cover_changes: EventQueue<Timestamped<CoverState>>,
    gestures: EventQueue<Timestamped<Gesture>>,
    wake_rearm_pending: bool,
}

impl SlpiAdapter {
    pub async fn connect(profile: &SlpiProfile, timeout_per_request: Duration) -> io::Result<Self> {
        profile.reports.validate()?;
        let optional_rates = [&profile.gyroscope, &profile.magnetometer]
            .into_iter()
            .flatten()
            .map(|stream| stream.hz);
        if std::iter::once(profile.accelerometer_hz)
            .chain(optional_rates)
            .any(|hz| !hz.is_finite() || hz <= 0.0)
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid SLPI sample rate",
            ));
        }

        let mut connection = SlpiConnection::connect(timeout_per_request).await?;
        let sensors = Sensors::discover(&mut connection, profile, timeout_per_request).await?;
        let measurements =
            Measurements::discover(&mut connection, &profile.measurements, timeout_per_request)
                .await?;
        let (moving, stationary) = if let Some(channel) = &profile.measurements.psmd {
            let (moving, stationary) = tokio::try_join!(
                MotionSubscription::connect(
                    channel,
                    true,
                    profile.reports.psmd,
                    timeout_per_request
                ),
                MotionSubscription::connect(
                    channel,
                    false,
                    profile.reports.psmd,
                    timeout_per_request
                )
            )?;
            (Some(moving), Some(stationary))
        } else {
            (None, None)
        };
        timeout(timeout_per_request, async {
            connection
                .send_request(&stream(sensors.acceleration, profile.accelerometer_hz))
                .await?;
            for (sensor, profile) in [
                (sensors.gyroscope, &profile.gyroscope),
                (sensors.magnetometer, &profile.magnetometer),
            ] {
                if let (Some(sensor), Some(profile)) = (sensor, profile) {
                    connection.send_request(&stream(sensor, profile.hz)).await?;
                }
            }
            for sensor in [sensors.cover, sensors.wake, sensors.tilt] {
                connection.send_request(&on_change(sensor)).await?;
            }
            measurements.subscribe(&mut connection).await
        })
        .await
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "sending SLPI subscriptions timed out",
            )
        })??;

        Ok(Self {
            connection,
            reports: profile.reports.clone(),
            sensors,
            measurements,
            moving,
            stationary,
            timestamp_hz: profile.timestamp_hz,
            buffer: vec![0; MAX_QMI_DATAGRAM],
            acceleration: EventQueue::default(),
            angular_velocity: EventQueue::default(),
            magnetic_field: EventQueue::default(),
            cover: None,
            cover_changes: EventQueue::default(),
            gestures: EventQueue::default(),
            wake_rearm_pending: false,
        })
    }

    pub async fn pump(&mut self) -> io::Result<()> {
        self.rearm_wake().await?;
        tokio::select! {
            batch = self.connection.receive_batch(&mut self.buffer) => {
                if let Some(batch) = batch? {
                    self.accept(batch)?;
                }
            }
            motion = next_motion(self.moving.as_mut()) => self.accept_motion(motion?),
            motion = next_motion(self.stationary.as_mut()) => self.accept_motion(motion?),
        }
        self.rearm_wake().await
    }

    async fn rearm_wake(&mut self) -> io::Result<()> {
        if self.wake_rearm_pending {
            self.connection
                .send_request(&on_change(self.sensors.wake))
                .await?;
            self.wake_rearm_pending = false;
        }
        if let Some(sensor) = self.measurements.pending_rearm {
            self.connection.send_request(&on_change(sensor)).await?;
            self.measurements.pending_rearm = None;
        }
        Ok(())
    }

    fn accept_motion(&mut self, reports: Vec<(u64, Motion)>) {
        for (ticks, value) in reports {
            self.measurements.motion.push(Timestamped {
                timestamp: timestamp(ticks, self.timestamp_hz),
                value,
            });
        }
    }

    fn accept(&mut self, batch: EventBatch) -> io::Result<()> {
        if self.measurements.accept(&batch, &self.reports, |ticks| {
            timestamp(ticks, self.timestamp_hz)
        })? {
            return Ok(());
        }
        for event in batch.events {
            let timestamp = timestamp(event.timestamp, self.timestamp_hz);
            match (batch.sensor, event.message) {
                (id, message)
                    if id == self.sensors.acceleration
                        || Some(id) == self.sensors.gyroscope
                        || Some(id) == self.sensors.magnetometer =>
                {
                    let format = if id == self.sensors.acceleration {
                        &self.reports.acceleration
                    } else if Some(id) == self.sensors.gyroscope {
                        &self.reports.gyroscope
                    } else {
                        &self.reports.magnetometer
                    };
                    if message != format.message {
                        continue;
                    }
                    let Some(values) = format.select(&event.payload)? else {
                        continue;
                    };
                    let [x, y, z] = values.as_slice() else {
                        continue;
                    };
                    if id == self.sensors.acceleration {
                        self.acceleration.push(Timestamped {
                            timestamp,
                            value: Acceleration {
                                x_mps2: *x,
                                y_mps2: *y,
                                z_mps2: *z,
                            },
                        });
                    } else if Some(id) == self.sensors.gyroscope {
                        self.angular_velocity.push(Timestamped {
                            timestamp,
                            value: AngularVelocity {
                                x_rad_s: *x,
                                y_rad_s: *y,
                                z_rad_s: *z,
                            },
                        });
                    } else {
                        self.magnetic_field.push(Timestamped {
                            timestamp,
                            value: MagneticField {
                                x_ut: *x,
                                y_ut: *y,
                                z_ut: *z,
                            },
                        });
                    }
                }
                (id, message)
                    if id == self.sensors.cover && message == self.reports.cover.message =>
                {
                    let code = integer(&event.payload, self.reports.cover.field)?
                        .map(|value| value as i32);
                    let value = match code {
                        Some(code) if code == self.reports.cover.open => CoverState::Open,
                        Some(code) if code == self.reports.cover.closed => CoverState::Closed,
                        _ => continue,
                    };
                    let sample = Timestamped { timestamp, value };
                    if self.cover.is_some_and(|previous| previous.value != value) {
                        self.cover_changes.push(sample);
                    }
                    self.cover = Some(sample);
                }
                (id, message) if id == self.sensors.wake && message == self.reports.wake => {
                    self.gestures.push(Timestamped {
                        timestamp,
                        value: Gesture::Wake,
                    });
                    self.wake_rearm_pending = true;
                }
                (id, message) if id == self.sensors.tilt && message == self.reports.tilt => {
                    self.gestures.push(Timestamped {
                        timestamp,
                        value: Gesture::Tilt,
                    });
                }
                _ => {}
            }
        }
        Ok(())
    }
}

impl AccelerometerPort for SlpiAdapter {
    type Error = io::Error;

    fn next_acceleration(&mut self) -> io::Result<Option<Timestamped<Acceleration>>> {
        self.acceleration.next()
    }
}

impl GyroscopePort for SlpiAdapter {
    type Error = io::Error;

    fn next_angular_velocity(&mut self) -> io::Result<Option<Timestamped<AngularVelocity>>> {
        self.angular_velocity.next()
    }
}

impl MagnetometerPort for SlpiAdapter {
    type Error = io::Error;

    fn next_magnetic_field(&mut self) -> io::Result<Option<Timestamped<MagneticField>>> {
        self.magnetic_field.next()
    }
}

impl CoverPort for SlpiAdapter {
    type Error = io::Error;

    fn cover_state(&self) -> io::Result<Option<Timestamped<CoverState>>> {
        Ok(self.cover)
    }

    fn next_cover_change(&mut self) -> io::Result<Option<Timestamped<CoverState>>> {
        self.cover_changes.next()
    }
}

impl GesturePort for SlpiAdapter {
    type Error = io::Error;

    fn next_gesture(&mut self) -> io::Result<Option<Timestamped<Gesture>>> {
        self.gestures.next()
    }
}

impl MotionPort for SlpiAdapter {
    type Error = io::Error;

    fn next_motion(&mut self) -> io::Result<Option<Timestamped<Motion>>> {
        self.measurements.motion.next()
    }
}

impl PedometerPort for SlpiAdapter {
    type Error = io::Error;

    fn next_step_count(&mut self) -> io::Result<Option<Timestamped<StepCount>>> {
        self.measurements.steps.next()
    }
}

impl OrientationPort for SlpiAdapter {
    type Error = io::Error;

    fn next_orientation(&mut self) -> io::Result<Option<Timestamped<Orientation>>> {
        self.measurements.orientation.next()
    }
}

impl RotationPort for SlpiAdapter {
    type Error = io::Error;

    fn next_rotation(
        &mut self,
        reference: RotationReference,
    ) -> io::Result<Option<Timestamped<Rotation>>> {
        self.measurements.rotation_queue(reference).next()
    }
}

impl GravityPort for SlpiAdapter {
    type Error = io::Error;

    fn next_gravity(&mut self) -> io::Result<Option<Timestamped<GravitySample>>> {
        self.measurements.gravity.next()
    }
}

impl ColorPort for SlpiAdapter {
    type Error = io::Error;

    fn next_color(&mut self) -> io::Result<Option<Timestamped<ColorSample>>> {
        self.measurements.color.next()
    }
}

impl IlluminancePort for SlpiAdapter {
    type Error = io::Error;

    fn next_illuminance(&mut self) -> io::Result<Option<Timestamped<Illuminance>>> {
        self.measurements.light.next()
    }
}

fn timestamp(ticks: u64, frequency: NonZeroU32) -> Duration {
    let hz = u64::from(frequency.get());
    Duration::new(ticks / hz, ((ticks % hz) * 1_000_000_000 / hz) as u32)
}
