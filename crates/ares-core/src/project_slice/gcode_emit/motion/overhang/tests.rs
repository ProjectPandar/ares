use super::speed_for_distance;

#[test]
fn task22o50_speed_interpolation_uses_source_float_precision() {
    let sections = [
        (0.042_f32, 200.0),
        (0.105_f32, 200.0),
        (0.21_f32, 50.0),
        (0.315_f32, 30.0),
        (0.3654_f32, 10.0),
        (0.42_f32, 50.0),
    ];

    assert_eq!(speed_for_distance(0.10535, &sections, 200.0), 200.0);
}
