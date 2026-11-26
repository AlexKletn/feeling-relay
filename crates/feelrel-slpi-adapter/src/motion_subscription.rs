use std::io;
use std::time::Duration;

use feelrel_ports::Motion;

use crate::{
    SlpiConnection, Suid, connection::MAX_QMI_DATAGRAM, report_profile::PsmdFormat, sns::psmd,
};

pub(super) struct MotionSubscription {
    connection: SlpiConnection,
    sensor: Suid,
    motion: Motion,
    report: u32,
    buffer: Vec<u8>,
}

impl MotionSubscription {
    pub async fn connect(
        channel: &str,
        moving: bool,
        format: PsmdFormat,
        wait: Duration,
    ) -> io::Result<Self> {
        let mut connection = SlpiConnection::connect(wait).await?;
        let sensor = connection.discover_channel(channel, wait).await?;
        tokio::time::timeout(wait, connection.send_request(&psmd(sensor, format, moving)))
            .await
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::TimedOut,
                    "sending PSMD subscription timed out",
                )
            })??;
        Ok(Self {
            connection,
            sensor,
            motion: if moving {
                Motion::Moving
            } else {
                Motion::Stationary
            },
            report: format.message,
            buffer: vec![0; MAX_QMI_DATAGRAM],
        })
    }

    pub async fn next(&mut self) -> io::Result<Vec<(u64, Motion)>> {
        if let Some(batch) = self.connection.receive_batch(&mut self.buffer).await?
            && batch.sensor == self.sensor
        {
            return Ok(batch
                .events
                .into_iter()
                .filter(|event| event.message == self.report)
                .map(|event| (event.timestamp, self.motion))
                .collect());
        }
        Ok(Vec::new())
    }
}

pub(super) async fn next_motion(
    subscription: Option<&mut MotionSubscription>,
) -> io::Result<Vec<(u64, Motion)>> {
    if let Some(subscription) = subscription {
        subscription.next().await
    } else {
        std::future::pending().await
    }
}
