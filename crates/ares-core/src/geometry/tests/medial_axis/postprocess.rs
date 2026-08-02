use crate::geometry::{ExPolygon, Point, Polygon, ThickPolyline, medial_axis::postprocess};

fn rectangle() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(1_000, 0),
            Point::new(1_000, 400),
            Point::new(0, 400),
        ]),
        Vec::new(),
    )
}

fn rectangle_with_hole() -> ExPolygon {
    ExPolygon::new(
        rectangle().contour().clone(),
        vec![Polygon::new(vec![
            Point::new(300, 100),
            Point::new(300, 300),
            Point::new(700, 300),
            Point::new(700, 100),
        ])],
    )
}

fn concave() -> ExPolygon {
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(400, 0),
            Point::new(400, 100),
            Point::new(100, 100),
            Point::new(100, 400),
            Point::new(0, 400),
        ]),
        Vec::new(),
    )
}

#[test]
fn task22o13_postprocess_empty_is_exactly_empty() {
    assert!(postprocess(&rectangle(), Vec::new(), 1_000.0, 100.0).is_empty());
}

#[test]
fn task22o13_endpoint_extension_uses_two_point_midpoint() {
    let input = ThickPolyline {
        points: vec![Point::new(500, 100), Point::new(500, 300)],
        width: vec![10.0, 10.0],
        endpoints: (true, false),
    };
    let output = postprocess(&rectangle(), vec![input], 1_000.0, 1.0);
    assert_eq!(output.len(), 1);
    assert_eq!(
        output[0].points,
        vec![Point::new(500, 0), Point::new(500, 300)]
    );
    assert_eq!(
        output[0]
            .width
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![0x4024_0000_0000_0000, 0x4024_0000_0000_0000]
    );
    assert_eq!(output[0].endpoints, (true, false));
}

#[test]
fn task22o13_endpoint_normalization_keeps_sqrt_order_and_truncating_eigen_cast() {
    let input = ThickPolyline {
        points: vec![
            Point::new(500, 200),
            Point::new(600, 300),
            Point::new(700, 300),
        ],
        width: vec![6.0, 6.0, 6.0, 6.0],
        endpoints: (true, false),
    };
    let output = postprocess(&rectangle(), vec![input], 1_000.0, 1.0);
    assert_eq!(
        output[0].points,
        vec![
            Point::new(300, 0),
            Point::new(600, 300),
            Point::new(700, 300),
        ]
    );
}

#[test]
fn task22o13_concave_contour_extension_uses_first_intersection_order() {
    let input = ThickPolyline {
        points: vec![
            Point::new(200, 50),
            Point::new(300, 50),
            Point::new(350, 50),
        ],
        width: vec![4.0, 4.0, 4.0, 4.0],
        endpoints: (true, false),
    };
    let output = postprocess(&concave(), vec![input], 1_000.0, 1.0);
    assert_eq!(
        output[0].points,
        vec![Point::new(0, 50), Point::new(300, 50), Point::new(350, 50)]
    );
}

#[test]
fn task22o13_hole_boundary_suppresses_endpoint_extension() {
    let input = ThickPolyline {
        points: vec![Point::new(300, 200), Point::new(200, 200)],
        width: vec![8.0, 8.0],
        endpoints: (true, false),
    };
    let output = postprocess(&rectangle_with_hole(), vec![input], 500.0, 1.0);
    assert_eq!(
        output[0].points,
        vec![Point::new(300, 200), Point::new(200, 200)]
    );
}

#[test]
fn task22o13_short_removal_triggers_ordered_greedy_reconnect() {
    let short = ThickPolyline {
        points: vec![Point::new(10, 10), Point::new(15, 10)],
        width: vec![1.0, 1.0],
        endpoints: (true, true),
    };
    let left = ThickPolyline {
        points: vec![Point::new(0, 200), Point::new(50, 200)],
        width: vec![1.0, 2.0],
        endpoints: (false, false),
    };
    let right = ThickPolyline {
        points: vec![Point::new(50, 200), Point::new(100, 200)],
        width: vec![3.0, 10.0],
        endpoints: (false, false),
    };
    let output = postprocess(&rectangle(), vec![short, left, right], 0.0, 1.0);
    assert_eq!(output.len(), 1);
    assert_eq!(
        output[0].points,
        vec![
            Point::new(0, 200),
            Point::new(50, 200),
            Point::new(100, 200)
        ]
    );
    assert_eq!(
        output[0]
            .width
            .iter()
            .map(|value| value.to_bits())
            .collect::<Vec<_>>(),
        vec![
            0x3ff0_0000_0000_0000,
            0x4000_0000_0000_0000,
            0x4008_0000_0000_0000,
            0x4024_0000_0000_0000,
        ]
    );
}

#[test]
fn task22o13_output_order_and_exact_width_bits_are_stable_without_removal() {
    let first = ThickPolyline {
        points: vec![Point::new(100, 100), Point::new(200, 100)],
        width: vec![1.25, 2.5],
        endpoints: (false, false),
    };
    let second = ThickPolyline {
        points: vec![Point::new(700, 300), Point::new(800, 300)],
        width: vec![3.75, 5.0],
        endpoints: (false, false),
    };
    let output = postprocess(&rectangle(), vec![first, second], 0.0, 1.0);
    assert_eq!(output[0].points[0], Point::new(100, 100));
    assert_eq!(output[1].points[0], Point::new(700, 300));
    assert_eq!(output[0].width[0].to_bits(), 0x3ff4_0000_0000_0000);
    assert_eq!(output[1].width[1].to_bits(), 0x4014_0000_0000_0000);
}
