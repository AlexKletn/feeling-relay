use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use feelrel_ports::{
    ColorSample, GravitySample, Illuminance, Orientation, Rotation, RotationReference, StepCount,
};

use super::iio_vector::scaled_word;

pub struct IioMeasurements {
    illuminance: File,
    color: File,
    gravity: File,
    rotation: File,
    game_rotation: File,
    geomagnetic_rotation: File,
    orientation: File,
    steps: File,
}

impl IioMeasurements {
    pub fn open(directory: &Path) -> io::Result<Self> {
        let open = |name| OpenOptions::new().write(true).open(directory.join(name));
        Ok(Self {
            illuminance: open("feelrel-iio-light")?,
            color: open("feelrel-iio-rgb")?,
            gravity: open("feelrel-iio-gravity")?,
            rotation: open("feelrel-iio-rotation")?,
            game_rotation: open("feelrel-iio-game-rotation")?,
            geomagnetic_rotation: open("feelrel-iio-geomagnetic-rotation")?,
            orientation: open("feelrel-iio-orientation")?,
            steps: open("feelrel-iio-steps")?,
        })
    }

    pub fn illuminance(&mut self, value: Illuminance) -> io::Result<()> {
        write_scaled(&mut self.illuminance, [(value.lux, 1000.0)])
    }

    pub fn color(&mut self, value: ColorSample) -> io::Result<()> {
        write_scaled(
            &mut self.color,
            [
                (value.temperature_kelvin, 1000.0),
                (value.clear, 1.0),
                (value.red, 1.0),
                (value.green, 1.0),
                (value.blue, 1.0),
                (value.white, 1.0),
                (value.integration_ms, 1000.0),
                (value.gain, 1000.0),
            ],
        )
    }

    pub fn gravity(&mut self, value: GravitySample) -> io::Result<()> {
        let [gravity, linear] = [value.gravity, value.linear_acceleration];
        write_scaled(
            &mut self.gravity,
            [
                (gravity.x_mps2, 1e6),
                (gravity.y_mps2, 1e6),
                (gravity.z_mps2, 1e6),
                (linear.x_mps2, 1e6),
                (linear.y_mps2, 1e6),
                (linear.z_mps2, 1e6),
            ],
        )
    }

    pub fn rotation(&mut self, reference: RotationReference, value: Rotation) -> io::Result<()> {
        let device = match reference {
            RotationReference::Fused => &mut self.rotation,
            RotationReference::Game => &mut self.game_rotation,
            RotationReference::Geomagnetic => &mut self.geomagnetic_rotation,
        };
        write_scaled(
            device,
            [
                (value.x, 1e6),
                (value.y, 1e6),
                (value.z, 1e6),
                (value.w, 1e6),
            ],
        )
    }

    pub fn orientation(&mut self, value: Orientation) -> io::Result<()> {
        let degrees: i32 = match value {
            Orientation::Unknown => -1,
            Orientation::Upright => 0,
            Orientation::Counterclockwise => 90,
            Orientation::UpsideDown => 180,
            Orientation::Clockwise => 270,
        };
        self.orientation.write_all(&degrees.to_le_bytes())
    }

    pub fn steps(&mut self, value: StepCount) -> io::Result<()> {
        self.steps.write_all(&value.steps.to_le_bytes())
    }
}

fn write_scaled<const N: usize>(device: &mut File, fields: [(f32, f64); N]) -> io::Result<()> {
    let mut words = [[0; 4]; N];
    for ((value, factor), bytes) in fields.into_iter().zip(&mut words) {
        *bytes = scaled_word(value, factor)?.to_le_bytes();
    }
    device.write_all(words.as_flattened())
}
