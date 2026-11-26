use super::*;
use prost::encoding::{float, uint32};

#[test]
fn uses_the_profile_field_order_and_per_value_scale() {
    let format = VectorFormat {
        message: 900,
        field: 3,
        order: vec![2, 0, 1],
        scales: Some(vec![1.0, -1.0, 0.5]),
    };
    let mut packet = Vec::new();
    uint32::encode(1, &123, &mut packet);
    float::encode_packed(3, &[3.0, 4.0, 5.0], &mut packet);
    assert_eq!(format.select(&packet).unwrap(), Some(vec![5.0, -3.0, 2.0]));
    let mut incomplete = Vec::new();
    float::encode_packed(3, &[3.0, 4.0], &mut incomplete);
    assert_eq!(format.select(&incomplete).unwrap(), None);
}
