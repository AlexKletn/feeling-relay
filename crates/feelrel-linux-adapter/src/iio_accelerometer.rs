use std::io;
use std::path::Path;

use feelrel_ports::Acceleration;

use super::iio_vector::IioVector;

pub struct IioAccelerometer(IioVector);

impl IioAccelerometer {
    pub fn open(path: &Path) -> io::Result<Self> {
        IioVector::open(path).map(Self)
    }

    pub fn publish(&mut self, sample: Acceleration) -> io::Result<()> {
        self.0
            .publish([sample.x_mps2, sample.y_mps2, sample.z_mps2], 1_000_000.0)
    }
}
