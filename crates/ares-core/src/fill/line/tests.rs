use super::*;

#[test]
fn shared_inset_erases_narrow_domains_unless_overlap_restores_them() {
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let point = |x, y| {
            Point::new(
                scale.checked_scale(x).unwrap(),
                scale.checked_scale(y).unwrap(),
            )
        };
        let domain = ExPolygon::new(
            Polygon::new(vec![
                point(0.0, 0.0),
                point(0.1, 0.0),
                point(0.1, 5.0),
                point(0.0, 5.0),
            ]),
            Vec::new(),
        );
        let mut params = LineFillParams {
            dont_alternate_fill_direction: false,
            spacing: 0.45,
            overlap: 0.0,
            density: 0.15,
            angle: -std::f32::consts::FRAC_PI_2,
            layer_index: 0,
            thickness_layers: 1,
            fixed_angle: true,
            bridge_angle: None,
            reference: point(0.0, 0.0),
            dont_adjust: false,
        };
        assert!(fill_line(&domain, params, scale).unwrap().is_empty());
        params.overlap = 0.225;
        assert!(!fill_line(&domain, params, scale).unwrap().is_empty());
    }
}

#[test]
fn clipping_and_connection_use_the_rotated_surface_frame() {
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let point = |x, y| {
            Point::new(
                scale.checked_scale(x).unwrap(),
                scale.checked_scale(y).unwrap(),
            )
        };
        let domain = ExPolygon::new(
            Polygon::new(vec![
                point(20.0, 10.0),
                point(40.0, 10.0),
                point(40.0, 20.0),
                point(20.0, 20.0),
            ]),
            Vec::new(),
        );
        for angle in [0.0, 0.3, 0.8] {
            let paths = fill_line(
                &domain,
                LineFillParams {
                    dont_alternate_fill_direction: false,
                    spacing: 0.45,
                    overlap: 0.0,
                    density: 0.15,
                    angle,
                    layer_index: 0,
                    thickness_layers: 1,
                    fixed_angle: true,
                    bridge_angle: None,
                    reference: point(30.0, 15.0),
                    dont_adjust: false,
                },
                scale,
            )
            .unwrap();
            assert!(!paths.is_empty(), "angle={angle}, scale={scale:?}");
            // FillLine chains compatible adjacent columns inside its expanded
            // rotated domain; concatenation must survive translation too.
            assert!(
                paths.iter().any(|path| path.points().len() > 2),
                "no connected columns: angle={angle}, scale={scale:?}"
            );
            for p in paths.iter().flat_map(|line| line.points()) {
                assert!((19.8..=40.2).contains(&scale.unscale(p.x())), "{p:?}");
                assert!((9.8..=20.2).contains(&scale.unscale(p.y())), "{p:?}");
            }
        }
    }
}
