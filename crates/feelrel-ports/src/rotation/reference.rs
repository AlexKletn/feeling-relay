#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RotationReference {
    Fused,
    Game,
    Geomagnetic,
}
