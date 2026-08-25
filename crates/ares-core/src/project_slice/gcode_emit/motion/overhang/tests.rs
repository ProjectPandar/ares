use super::{scale_trunc, speed_for_distance};
use crate::geometry::CoordinateScale;

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

#[test]
fn task22o137_processed_points_truncate_like_source_scaled_vectors() {
    let scale = CoordinateScale::Normal;

    assert_eq!(scale_trunc(5.006_295_746_287_667, scale), 5_006_295);
    assert_eq!(scale_trunc(-3.623_527_714_016_064_7, scale), -3_623_527);
}
