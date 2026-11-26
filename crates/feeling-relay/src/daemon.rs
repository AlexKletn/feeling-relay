use std::io;
use std::time::Duration;

use crate::measurement_relay::MeasurementRelay;
use feelrel_linux_adapter::{IioAccelerometer, IioGyroscope, IioMagnetometer};
use feelrel_ports::{AccelerometerPort, GyroscopePort, MagnetometerPort};
use feelrel_slpi_adapter::SlpiAdapter;
use tokio::time::{Instant, sleep, timeout_at};

use crate::{config::Config, input_relay::InputRelay, sample_deadline::SampleDeadline};

const RECONNECT_DELAY: Duration = Duration::from_secs(1);

pub(super) async fn run(config: Config) -> io::Result<()> {
    loop {
        let disconnect = run_session(&config).await?;
        eprintln!("feeling-relay: {disconnect}; retrying SLPI connection");
        sleep(RECONNECT_DELAY).await;
    }
}

async fn run_session(config: &Config) -> io::Result<io::Error> {
    let mut source = match SlpiAdapter::connect(&config.slpi, config.request_timeout()).await {
        Ok(source) => source,
        Err(error) if reconnectable(&error) => return Ok(error),
        Err(error) => return Err(error),
    };
    let mut input = InputRelay::open(&config.input)?;
    let mut accelerometer = config
        .iio_accelerometer_device
        .as_deref()
        .map(IioAccelerometer::open)
        .transpose()?;
    let mut gyroscope = config
        .iio_gyroscope_device
        .as_deref()
        .map(IioGyroscope::open)
        .transpose()?;
    let mut magnetometer = config
        .iio_magnetometer_device
        .as_deref()
        .map(IioMagnetometer::open)
        .transpose()?;
    let mut measurements = MeasurementRelay::open(config.iio_measurements_directory.as_deref())?;
    let mut deadline = SampleDeadline::new(Instant::now(), config.sample_timeout());
    eprintln!("feeling-relay: connected to SLPI");

    loop {
        match timeout_at(deadline.expires_at(), source.pump()).await {
            Ok(Ok(())) => {}
            Ok(Err(error)) if reconnectable(&error) => return Ok(error),
            Ok(Err(error)) => return Err(error),
            Err(_) => {
                return Ok(io::Error::new(
                    io::ErrorKind::TimedOut,
                    "SLPI accelerometer stopped reporting",
                ));
            }
        }
        while let Some(sample) = source.next_acceleration()? {
            deadline.record_sample(Instant::now());
            if let Some(publisher) = &mut accelerometer {
                publisher.publish(sample.value)?;
            }
        }
        while let Some(sample) = source.next_angular_velocity()? {
            if let Some(publisher) = &mut gyroscope {
                publisher.publish(sample.value)?;
            }
        }
        while let Some(sample) = source.next_magnetic_field()? {
            if let Some(publisher) = &mut magnetometer {
                publisher.publish(sample.value)?;
            }
        }
        measurements.publish(&mut source)?;
        input.publish(&mut source)?;
    }
}

fn reconnectable(error: &io::Error) -> bool {
    matches!(
        error.kind(),
        io::ErrorKind::TimedOut
            | io::ErrorKind::ConnectionAborted
            | io::ErrorKind::ConnectionReset
            | io::ErrorKind::ConnectionRefused
            | io::ErrorKind::NotConnected
            | io::ErrorKind::NetworkDown
            | io::ErrorKind::NetworkUnreachable
            | io::ErrorKind::HostUnreachable
            | io::ErrorKind::BrokenPipe
            | io::ErrorKind::NotFound
    ) || matches!(
        error.raw_os_error(),
        Some(libc::ENETRESET | libc::ENODEV | libc::EHOSTDOWN)
    )
}
