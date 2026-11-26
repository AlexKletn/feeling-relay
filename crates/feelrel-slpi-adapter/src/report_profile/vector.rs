use serde::Deserialize;

#[derive(Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct VectorFormat {
    pub message: u32,
    pub field: u32,
    pub order: Vec<usize>,
    pub scales: Option<Vec<f32>>,
}

impl VectorFormat {
    pub fn select(&self, payload: &[u8]) -> std::io::Result<Option<Vec<f32>>> {
        let values = crate::sns::float_values(payload, self.field)?;
        Ok(self
            .order
            .iter()
            .enumerate()
            .map(|(position, &index)| {
                let factor = self.scales.as_ref().map_or(1.0, |scales| scales[position]);
                values.get(index).map(|value| value * factor)
            })
            .collect())
    }
}

#[cfg(test)]
#[path = "vector.tests.rs"]
mod tests;
