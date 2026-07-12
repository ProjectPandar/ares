use crate::{BrimOptions, BrimType, Contour, LayerContours, Point2, generate_brims};

#[test]
fn active_efc_outline_shrinks_outer_brim_base_before_loop_expansion() {
    let contours = [LayerContours::new(0, 0.2, vec![square(0.0, 0.0, 4.0, 4.0)])];
    let options =
        BrimOptions::new(0.4, 0.0, BrimType::OuterOnly).with_efc_outline_offset_mm(Some(0.2));

    let output = generate_brims(&contours, options, 0.4).unwrap();

    assert_eq!(output[0].paths().len(), 1);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.2, -0.2),
            Point2::new(4.2, -0.2),
            Point2::new(4.2, 4.2),
            Point2::new(-0.2, 4.2),
        ]
    );
}

#[test]
fn inactive_efc_outline_keeps_raw_outer_brim_base() {
    let contours = [LayerContours::new(0, 0.2, vec![square(0.0, 0.0, 4.0, 4.0)])];
    let options = BrimOptions::new(0.4, 0.0, BrimType::OuterOnly);

    let output = generate_brims(&contours, options, 0.4).unwrap();

    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.4, -0.4),
            Point2::new(4.4, -0.4),
            Point2::new(4.4, 4.4),
            Point2::new(-0.4, 4.4),
        ]
    );
}

#[test]
fn collapsed_efc_outline_skips_outer_brim_path() {
    let contours = [LayerContours::new(0, 0.2, vec![square(0.0, 0.0, 0.4, 0.4)])];
    let options =
        BrimOptions::new(0.4, 0.0, BrimType::OuterOnly).with_efc_outline_offset_mm(Some(0.2));

    let output = generate_brims(&contours, options, 0.4).unwrap();

    assert!(output[0].paths().is_empty());
}

#[test]
fn combined_outer_brim_uses_efc_adjusted_bounds() {
    let contours = [LayerContours::new(
        0,
        0.2,
        vec![square(0.0, 0.0, 2.0, 2.0), square(4.0, 0.0, 6.0, 2.0)],
    )];
    let options = BrimOptions::new(0.4, 0.0, BrimType::OuterOnly)
        .with_combine_brims(true)
        .with_efc_outline_offset_mm(Some(0.2));

    let output = generate_brims(&contours, options, 0.4).unwrap();

    assert_eq!(output[0].paths().len(), 1);
    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-0.2, -0.2),
            Point2::new(6.2, -0.2),
            Point2::new(6.2, 2.2),
            Point2::new(-0.2, 2.2),
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
