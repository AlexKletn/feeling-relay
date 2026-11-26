#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorSample {
    pub temperature_kelvin: f32,
    pub clear: f32,
    pub red: f32,
    pub green: f32,
    pub blue: f32,
    pub white: f32,
    pub integration_ms: f32,
    pub gain: f32,
}
