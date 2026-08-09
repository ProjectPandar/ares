use super::*;
use crate::geometry::{CoordinateScale, diff_pl};

#[test]
fn task22o39_empty_anchors_preserve_complete_contour_hole_line_order_and_angle_bits() {
    let geometry = expolygon(
        &[(0, 0), (1200, 0), (1200, 800), (0, 800)],
        vec![polygon(&[(400, 200), (400, 600), (800, 600), (800, 200)])],
    );
    let expected = manual(&geometry, &[], CoordinateScale::Normal).unwrap();
    assert!(expected.expanded.is_empty());
    assert_eq!(
        fragment_points(&expected.fragments),
        vec![
            vec![(0, 0), (1200, 0), (1200, 800), (0, 800), (0, 0)],
            vec![(400, 200), (800, 200), (800, 600), (400, 600), (400, 200)],
        ]
    );
    assert_eq!(
        line_points(&expected.lines),
        vec![
            ((0, 0), (1200, 0)),
            ((1200, 0), (1200, 800)),
            ((1200, 800), (0, 800)),
            ((0, 800), (0, 0)),
            ((400, 200), (800, 200)),
            ((800, 200), (800, 600)),
            ((800, 600), (400, 600)),
            ((400, 600), (400, 200)),
        ]
    );
    assert_eq!(expected.direction.0.to_bits(), (-1.0_f64).to_bits());
    assert_eq!(expected.direction.1.to_bits(), (-0.0_f64).to_bits());
    assert_eq!(expected.cost.to_bits(), 2400.0_f64.to_bits());
    assert_eq!(expected.angle.to_bits(), 0.0_f64.to_bits());

    let mut bridges = vec![bridge(geometry, Some(-7.0))];
    DETECT(
        &[],
        &mut bridges,
        &[zone(Vec::new())],
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(angles(&bridges), vec![Some(0.0_f64.to_bits())]);
}

#[test]
fn task22o39_non_recombining_open_difference_retains_pinned_fragment_and_line_topology() {
    let geometry = rectangle(0, 0, 1000, 1000);
    let anchor = rectangle(400, -200, 600, 200).contour().clone();
    let expected = manual(
        &geometry,
        std::slice::from_ref(&anchor),
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(
        polygon_points(&expected.expanded),
        vec![vec![(700, 300), (300, 300), (300, -300), (700, -300)]]
    );
    assert_eq!(
        fragment_points(&expected.fragments),
        vec![
            vec![(700, 0), (1000, 0), (1000, 1000), (0, 1000), (0, 0)],
            vec![(0, 0), (300, 0)],
        ]
    );
    assert_eq!(
        line_points(&expected.lines),
        vec![
            ((700, 0), (1000, 0)),
            ((1000, 0), (1000, 1000)),
            ((1000, 1000), (0, 1000)),
            ((0, 1000), (0, 0)),
            ((0, 0), (300, 0)),
        ]
    );
    let recombined = diff_pl(&bridge_polygons(&geometry), &expected.expanded).unwrap();
    assert_eq!(
        fragment_points(&recombined),
        vec![vec![
            (700, 0),
            (1000, 0),
            (1000, 1000),
            (0, 1000),
            (0, 0),
            (300, 0),
        ]]
    );
    assert_eq!(expected.direction, (0.0, -1.0));
    assert_eq!(expected.cost.to_bits(), 1600.0_f64.to_bits());
    assert_eq!(expected.angle.to_bits(), 0x3ff9_21fb_5444_2d18);

    let anchors = vec![seed(0, 0, &[(777, 888)]), seed(0, 0, &[(999, 111)])];
    let mut bridges = vec![bridge(geometry, None)];
    DETECT(
        &anchors,
        &mut bridges,
        &[zone(vec![rectangle(400, -200, 600, 200)])],
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(angles(&bridges), vec![Some(0x3ff9_21fb_5444_2d18)]);
}

#[test]
fn task22o39_normal_and_large_bed_freeze_f64_to_f32_epsilon_and_forward_scale() {
    let geometry = expolygon(
        &[(0, 0), (1200, 0), (1200, 900), (0, 900)],
        vec![polygon(&[(300, 300), (300, 600), (900, 600), (900, 300)])],
    );
    let anchor = rectangle(450, -100, 750, 150).contour().clone();
    let normal = manual(
        &geometry,
        std::slice::from_ref(&anchor),
        CoordinateScale::Normal,
    )
    .unwrap();
    let large = manual(
        &geometry,
        std::slice::from_ref(&anchor),
        CoordinateScale::LargeBed,
    )
    .unwrap();
    assert_eq!((1e-4_f64 / CoordinateScale::Normal.factor()) as f32, 100.0);
    assert_eq!((1e-4_f64 / CoordinateScale::LargeBed.factor()) as f32, 10.0);
    assert_eq!(
        polygon_points(&normal.expanded),
        vec![vec![(850, 250), (350, 250), (350, -200), (850, -200)]]
    );
    assert_eq!(
        polygon_points(&large.expanded),
        vec![vec![(760, 160), (440, 160), (440, -110), (760, -110)]]
    );
    assert_ne!(
        fragment_points(&normal.fragments),
        fragment_points(&large.fragments)
    );
    assert_eq!(large.angle.to_bits(), 0.0_f64.to_bits());

    let anchor_with_hole = expolygon(
        &[(50, 650), (250, 650), (250, 850), (50, 850)],
        vec![polygon(&[(100, 700), (100, 800), (200, 800), (200, 700)])],
    );
    let selected = vec![
        anchor.clone(),
        anchor_with_hole.contour().clone(),
        anchor_with_hole.holes()[0].clone(),
    ];
    let complete = manual(&geometry, &selected, CoordinateScale::LargeBed).unwrap();
    assert_eq!(
        fragment_points(&complete.fragments),
        vec![
            vec![(760, 0), (1200, 0), (1200, 900), (0, 900), (0, 0)],
            vec![(300, 300), (900, 300), (900, 600), (300, 600), (300, 300)],
            vec![(0, 0), (440, 0)],
        ]
    );
    assert_eq!(
        line_points(&complete.lines),
        vec![
            ((760, 0), (1200, 0)),
            ((1200, 0), (1200, 900)),
            ((1200, 900), (0, 900)),
            ((0, 900), (0, 0)),
            ((300, 300), (900, 300)),
            ((900, 300), (900, 600)),
            ((900, 600), (300, 600)),
            ((300, 600), (300, 300)),
            ((0, 0), (440, 0)),
        ]
    );
    assert_eq!(complete.angle.to_bits(), 0.0_f64.to_bits());
    let zones = vec![
        zone(vec![rectangle(450, -100, 750, 150)]),
        zone(vec![anchor_with_hole]),
    ];
    let anchors = vec![seed(0, 0, &[(1, 2)]), seed(0, 1, &[(3, 4)])];
    let mut bridges = vec![bridge(geometry, None)];
    DETECT(&anchors, &mut bridges, &zones, CoordinateScale::LargeBed).unwrap();
    assert_eq!(angles(&bridges), vec![Some(0.0_f64.to_bits())]);
}
