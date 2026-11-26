use super::*;

#[test]
fn preserves_signed_axes_and_converts_microtesla_to_microgauss() {
    let frame = encode([1.25, -2.5, 0.0], 1_000_000.0).unwrap();
    assert_eq!(frame[..4], 1_250_000i32.to_le_bytes());
    assert_eq!(frame[4..8], (-2_500_000i32).to_le_bytes());
    let magnetic_frame = encode([25.0, 0.0, -25.0], 10_000.0).unwrap();
    assert_eq!(magnetic_frame[..4], 250_000i32.to_le_bytes());
    assert_eq!(magnetic_frame[8..], (-250_000i32).to_le_bytes());
    assert!(encode([0.0, 0.0, f32::NAN], 1_000_000.0).is_err());
    assert!(encode([0.0, 0.0, 3000.0], 1_000_000.0).is_err());
}
