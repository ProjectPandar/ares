use super::*;

#[test]
fn disabled_draft_shield_combined_skirt_uses_first_layer_brim_envelope() {
    let contours = vec![LayerContours::new(0, 0.2, vec![square(0.0, 0.0, 1.0, 1.0)])];
    let brims = vec![crate::LayerBrims::new(
        0,
        0.2,
        vec![
            crate::BrimPath::new(vec![
                Point2::new(-1.0, -1.0),
                Point2::new(2.0, -1.0),
                Point2::new(2.0, 2.0),
                Point2::new(-1.0, 2.0),
            ])
            .unwrap(),
        ],
    )];

    let output = generate_skirts_after_brims(
        &contours,
        &brims,
        SkirtOptions::new(1, 1.0, 1, 50.0),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-2.0, -2.0),
            Point2::new(3.0, -2.0),
            Point2::new(3.0, 3.0),
            Point2::new(-2.0, 3.0),
        ]
    );
}

#[test]
fn no_brim_envelope_preserves_combined_skirt_bounds() {
    let contours = vec![LayerContours::new(0, 0.2, vec![square(0.0, 0.0, 1.0, 1.0)])];
    let brims = vec![crate::LayerBrims::new(0, 0.2, Vec::new())];

    let output = generate_skirts_after_brims(
        &contours,
        &brims,
        SkirtOptions::new(1, 1.0, 1, 50.0),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-1.0, -1.0),
            Point2::new(2.0, -1.0),
            Point2::new(2.0, 2.0),
            Point2::new(-1.0, 2.0),
        ]
    );
}

#[test]
fn draft_shield_skirt_ignores_first_layer_brim_envelope() {
    let contours = vec![LayerContours::new(0, 0.2, vec![square(0.0, 0.0, 1.0, 1.0)])];
    let brims = vec![crate::LayerBrims::new(
        0,
        0.2,
        vec![
            crate::BrimPath::new(vec![
                Point2::new(-1.0, -1.0),
                Point2::new(2.0, -1.0),
                Point2::new(2.0, 2.0),
                Point2::new(-1.0, 2.0),
            ])
            .unwrap(),
        ],
    )];

    let output = generate_skirts_after_brims(
        &contours,
        &brims,
        SkirtOptions::new(1, 1.0, 1, 50.0).with_draft_shield(DraftShield::Enabled),
        0.4,
        1.0,
    )
    .unwrap();

    assert_eq!(
        output[0].paths()[0].points(),
        &[
            Point2::new(-1.0, -1.0),
            Point2::new(2.0, -1.0),
            Point2::new(2.0, 2.0),
            Point2::new(-1.0, 2.0),
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
