use std::io;
use std::path::Path;

use feelrel_linux_adapter::IioMeasurements;
use feelrel_ports::{
    ColorPort, GravityPort, IlluminancePort, OrientationPort, PedometerPort, RotationPort,
    RotationReference, Timestamped,
};
use feelrel_slpi_adapter::SlpiAdapter;

pub(super) struct MeasurementRelay(Option<IioMeasurements>);

impl MeasurementRelay {
    pub fn open(path: Option<&Path>) -> io::Result<Self> {
        Ok(Self(path.map(IioMeasurements::open).transpose()?))
    }

    pub fn publish(&mut self, source: &mut SlpiAdapter) -> io::Result<()> {
        drain(
            || source.next_illuminance(),
            &mut self.0,
            IioMeasurements::illuminance,
        )?;
        drain(|| source.next_color(), &mut self.0, IioMeasurements::color)?;
        drain(
            || source.next_gravity(),
            &mut self.0,
            IioMeasurements::gravity,
        )?;
        for reference in [
            RotationReference::Fused,
            RotationReference::Game,
            RotationReference::Geomagnetic,
        ] {
            drain(
                || source.next_rotation(reference),
                &mut self.0,
                |output, value| output.rotation(reference, value),
            )?;
        }
        drain(
            || source.next_orientation(),
            &mut self.0,
            IioMeasurements::orientation,
        )?;
        drain(
            || source.next_step_count(),
            &mut self.0,
            IioMeasurements::steps,
        )
    }
}

fn drain<T>(
    mut next: impl FnMut() -> io::Result<Option<Timestamped<T>>>,
    output: &mut Option<IioMeasurements>,
    mut publish: impl FnMut(&mut IioMeasurements, T) -> io::Result<()>,
) -> io::Result<()> {
    while let Some(sample) = next()? {
        if let Some(output) = output {
            publish(output, sample.value)?;
        }
    }
    Ok(())
}
