use std::io;
use std::time::Duration;

use crate::{SlpiConnection, SlpiProfile, StreamProfile, Suid};

pub(super) struct Sensors {
    pub acceleration: Suid,
    pub gyroscope: Option<Suid>,
    pub magnetometer: Option<Suid>,
    pub cover: Suid,
    pub wake: Suid,
    pub tilt: Suid,
}

impl Sensors {
    pub async fn discover(
        connection: &mut SlpiConnection,
        profile: &SlpiProfile,
        wait: Duration,
    ) -> io::Result<Self> {
        let acceleration = connection
            .discover_channel(&profile.accelerometer_channel, wait)
            .await?;
        let cover = connection
            .discover_channel(&profile.cover_channel, wait)
            .await?;
        let wake = connection
            .discover_channel(&profile.wake_gesture_channel, wait)
            .await?;
        let tilt = connection
            .discover_channel(&profile.tilt_channel, wait)
            .await?;
        Ok(Self {
            acceleration,
            gyroscope: discover_optional(connection, profile.gyroscope.as_ref(), wait).await?,
            magnetometer: discover_optional(connection, profile.magnetometer.as_ref(), wait)
                .await?,
            cover,
            wake,
            tilt,
        })
    }
}

async fn discover_optional(
    connection: &mut SlpiConnection,
    stream: Option<&StreamProfile>,
    wait: Duration,
) -> io::Result<Option<Suid>> {
    if let Some(stream) = stream {
        connection
            .discover_channel(&stream.channel, wait)
            .await
            .map(Some)
    } else {
        Ok(None)
    }
}
