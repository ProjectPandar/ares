use super::point_segment_distance;

#[test]
fn task22o129_segment_distance_clamps_to_endpoints() {
    assert_eq!(
        point_segment_distance((0.5, 1.0), (0.0, 0.0), (1.0, 0.0)),
        1.0
    );
    assert_eq!(
        point_segment_distance((2.0, 0.0), (0.0, 0.0), (1.0, 0.0)),
        1.0
    );
}
