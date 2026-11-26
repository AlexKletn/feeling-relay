use std::io;
use std::path::Path;

use feelrel_ports::MagneticField;

use super::iio_vector::IioVector;

pub struct IioMagnetometer(IioVector);

impl IioMagnetometer {
    pub fn open(path: &Path) -> io::Result<Self> {
        IioVector::open(path).map(Self)
    }

    pub fn publish(&mut self, sample: MagneticField) -> io::Result<()> {
        self.0
            .publish([sample.x_ut, sample.y_ut, sample.z_ut], 10_000.0)
    }
}
