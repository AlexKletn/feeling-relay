use super::*;

#[test]
fn keeps_rotation_references_separate() {
    let mut measurements = Measurements::default();
    let now = Duration::from_millis(47);
    measurements.accept_values(
        MeasurementKind::Rotation(RotationReference::Game),
        now,
        &[0.0, 0.6, 0.0, 0.8],
    );
    assert!(measurements.rotation.next().unwrap().is_none());
    assert!(measurements.geomagnetic_rotation.next().unwrap().is_none());
    let sample = measurements.game_rotation.next().unwrap().unwrap();
    assert_eq!(sample.timestamp, now);
    assert_eq!(sample.value.w, 0.8);
}

#[test]
fn decodes_color_and_gravity_fields_without_short_reports() {
    let mut measurements = Measurements::default();
    let now = Duration::from_millis(81);
    measurements.accept_values(
        MeasurementKind::Color,
        now,
        &[5200.0, 100.0, 32.0, 41.0, 38.0, 55.0, 20.0, 8.0],
    );
    measurements.accept_values(MeasurementKind::Color, now, &[5200.0, 100.0]);
    measurements.accept_values(
        MeasurementKind::Gravity,
        now,
        &[0.0, 1.0, 9.8, 0.2, -0.1, 0.3],
    );
    assert_eq!(
        measurements.color.next().unwrap().unwrap().value.white,
        55.0
    );
    assert!(measurements.color.next().unwrap().is_none());
    assert_eq!(
        measurements
            .gravity
            .next()
            .unwrap()
            .unwrap()
            .value
            .linear_acceleration
            .y_mps2,
        -0.1
    );
}
