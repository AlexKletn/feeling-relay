use std::fs::{File, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

pub(super) struct IioVector(File);

impl IioVector {
    pub fn open(path: &Path) -> io::Result<Self> {
        OpenOptions::new().write(true).open(path).map(Self)
    }

    pub fn publish(&mut self, axes: [f32; 3], factor: f64) -> io::Result<()> {
        self.0.write_all(&encode(axes, factor)?)
    }
}

fn encode(axes: [f32; 3], factor: f64) -> io::Result<[u8; 12]> {
    let mut frame = [0; 12];
    for (axis, field) in axes.into_iter().zip(frame.chunks_exact_mut(4)) {
        field.copy_from_slice(&scaled_word(axis, factor)?.to_le_bytes());
    }
    Ok(frame)
}

pub(super) fn scaled_word(value: f32, factor: f64) -> io::Result<i32> {
    let scaled = (f64::from(value) * factor).round();
    if !(f64::from(i32::MIN)..=f64::from(i32::MAX)).contains(&scaled) {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "measurement exceeds IIO range",
        ));
    }
    Ok(scaled as i32)
}

#[cfg(test)]
#[path = "iio_vector.tests.rs"]
mod tests;
