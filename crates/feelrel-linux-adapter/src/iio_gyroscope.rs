use std::io;
use std::path::Path;

use feelrel_ports::AngularVelocity;

use super::iio_vector::IioVector;

pub struct IioGyroscope(IioVector);

impl IioGyroscope {
    pub fn open(path: &Path) -> io::Result<Self> {
        IioVector::open(path).map(Self)
    }

    pub fn publish(&mut self, sample: AngularVelocity) -> io::Result<()> {
        self.0.publish(
            [sample.x_rad_s, sample.y_rad_s, sample.z_rad_s],
            1_000_000.0,
        )
    }
}
