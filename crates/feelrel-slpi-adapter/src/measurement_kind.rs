use feelrel_ports::RotationReference;

#[derive(Clone, Copy)]
pub(super) enum MeasurementKind {
    Illuminance,
    Color,
    Gravity,
    Rotation(RotationReference),
    Orientation,
    Pedometer,
    SignificantMotion,
}
