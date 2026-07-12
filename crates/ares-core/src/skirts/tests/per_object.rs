use super::*;

#[test]
fn per_object_skirt_type_generates_one_loop_per_outer_contour() {
    let contours = vec![LayerContours::new(
        0,
        0.2,
        vec![square(0.0, 0.0, 1.0, 1.0), square(3.0, 0.0, 4.0, 1.0)],
    )];

    let combined = generate_skirts(
        &contours,
        SkirtOptions::new(1, 1.0, 1, 50.0).with_skirt_type(SkirtType::Combined),
        0.4,
        1.0,
    )
    .unwrap();
    let per_object = generate_skirts(
        &contours,
        SkirtOptions::new(1, 1.0, 1, 50.0).with_skirt_type(SkirtType::PerObject),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(combined[0].paths().len(), 1);
    assert_eq!(
        combined[0].paths()[0].points(),
        &[
            Point2::new(-1.0, -1.0),
            Point2::new(5.0, -1.0),
            Point2::new(5.0, 2.0),
            Point2::new(-1.0, 2.0),
        ]
    );
    assert_eq!(per_object[0].paths().len(), 2);
    assert_eq!(
        per_object[0].paths()[0].points(),
        &[
            Point2::new(-1.0, -1.0),
            Point2::new(2.0, -1.0),
            Point2::new(2.0, 2.0),
            Point2::new(-1.0, 2.0),
        ]
    );
    assert_eq!(
        per_object[0].paths()[1].points(),
        &[
            Point2::new(2.0, -1.0),
            Point2::new(5.0, -1.0),
            Point2::new(5.0, 2.0),
            Point2::new(2.0, 2.0),
        ]
    );
}

#[test]
fn per_object_skirt_type_skips_inner_hole_contours() {
    let contours = vec![LayerContours::new(
        0,
        0.2,
        vec![square(0.0, 0.0, 4.0, 4.0), square(1.0, 1.0, 2.0, 2.0)],
    )];

    let output = generate_skirts(
        &contours,
        SkirtOptions::new(1, 1.0, 1, 50.0).with_skirt_type(SkirtType::PerObject),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(output[0].paths().len(), 1);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-1.0, -1.0),
            Point2::new(5.0, -1.0),
            Point2::new(5.0, 5.0),
            Point2::new(-1.0, 5.0),
        ]
    );
}

fn square(min_x: f64, min_y: f64, max_x: f64, max_y: f64) -> Contour {
    Contour::new(vec![
        Point2::new(min_x, min_y),
        Point2::new(max_x, min_y),
        Point2::new(max_x, max_y),
        Point2::new(min_x, max_y),
    ])
}
