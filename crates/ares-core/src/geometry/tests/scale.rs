use crate::geometry::CoordinateScale;
use crate::{Point2d, Point2dList};

fn area(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Point2dList {
    Point2dList(vec![Point2d::new(min_x, min_y), Point2d::new(max_x, max_y)])
}

#[test]
fn task22b_scale_selection_uses_printable_span_threshold_ksr_and_empty_default() {
    let threshold = area(-1_000.0, 20.0, 1_147.0, 60.0);
    let above_threshold = area(-1_000.0, 20.0, 1_147.000_001, 60.0);
    let ksr = area(0.0, 0.0, 256.0, 256.0);
    let empty = Point2dList(Vec::new());
    let unordered_y_dominant = Point2dList(vec![
        Point2d::new(10.0, 100.0),
        Point2d::new(20.0, -1_000.0),
        Point2d::new(15.0, 1_147.000_001),
        Point2d::new(12.0, 101.0),
    ]);

    assert_eq!(
        CoordinateScale::from_printable_area(&threshold).factor(),
        0.000_001
    );
    assert_eq!(
        CoordinateScale::from_printable_area(&above_threshold).factor(),
        0.000_01
    );
    assert_eq!(
        CoordinateScale::from_printable_area(&ksr).factor(),
        0.000_001
    );
    assert_eq!(
        CoordinateScale::from_printable_area(&empty).factor(),
        0.000_001
    );
    assert_eq!(
        CoordinateScale::from_printable_area(&unordered_y_dominant).factor(),
        0.000_01
    );
}

#[tokio::test(flavor = "current_thread")]
async fn task22b_request_local_scales_are_repeated_and_concurrently_isolated() {
    async fn observe(area: &Point2dList, expected_factor: f64) {
        for _ in 0..16 {
            let scale = CoordinateScale::from_printable_area(area);
            tokio::task::yield_now().await;
            assert_eq!(scale.factor(), expected_factor);
        }
    }

    let normal = area(0.0, 0.0, 2_147.0, 2_147.0);
    let large = area(0.0, 0.0, 2_147.001, 2_147.001);
    tokio::join!(observe(&normal, 0.000_001), observe(&large, 0.000_01));

    assert_eq!(
        CoordinateScale::from_printable_area(&normal).factor(),
        0.000_001
    );
    assert_eq!(
        CoordinateScale::from_printable_area(&large).factor(),
        0.000_01
    );
}
