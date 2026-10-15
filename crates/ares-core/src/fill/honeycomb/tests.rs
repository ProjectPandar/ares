use super::*;

fn surface(scale: CoordinateScale) -> ExPolygon {
    let h = scale.checked_scale(10.0).unwrap();
    ExPolygon::new(
        Polygon::new(vec![
            Point::new(-h, -h),
            Point::new(h, -h),
            Point::new(h, h),
            Point::new(-h, h),
        ]),
        Vec::new(),
    )
}

fn params(layer_index: usize) -> HoneycombFillParams {
    HoneycombFillParams {
        dont_alternate_fill_direction: false,
        spacing: 0.45,
        density: 0.15,
        multiline: 1,
        angle: 0.0,
        layer_index,
        thickness_layers: 1,
        fixed_angle: false,
        bridge_angle: None,
        anchor_length: 0.0,
        anchor_length_max: 0.0,
    }
}

#[test]
fn multiline_bundles_remain_inside_the_fill_domain() {
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let domain = surface(scale);
        for multiline in [2, 3, 4] {
            let mut config = params(0);
            config.multiline = multiline;
            let lines = fill_honeycomb(&domain, config, scale).unwrap();
            assert!(!lines.is_empty(), "multiline={multiline}, scale={scale:?}");
            let length: f64 = lines
                .iter()
                .flat_map(|line| line.points().windows(2))
                .map(|segment| {
                    let dx = scale.unscale(segment[1].x() - segment[0].x());
                    let dy = scale.unscale(segment[1].y() - segment[0].y());
                    dx.hypot(dy)
                })
                .sum();
            assert!(length > 60.0, "bundle lost its offset rings: {length}");
        }
    }
}

#[test]
fn lattice_repeats_after_three_layers() {
    let scale = CoordinateScale::Normal;
    let domain = surface(scale);
    let first = fill_honeycomb(&domain, params(0), scale).unwrap();
    let fourth = fill_honeycomb(&domain, params(3), scale).unwrap();
    assert!(!first.is_empty());
    assert_eq!(first, fourth);
    assert_ne!(first, fill_honeycomb(&domain, params(1), scale).unwrap());
}

#[test]
fn fractional_spacing_uses_integer_then_float_cache_arithmetic() {
    let scale = CoordinateScale::Normal;
    let mut config = params(0);
    config.spacing = 0.40707963705062866;
    config.angle = -std::f32::consts::FRAC_PI_2;
    config.fixed_angle = true;
    let domain = ExPolygon::new(
        Polygon::new(vec![
            Point::new(0, 0),
            Point::new(10_000_000, 0),
            Point::new(10_000_000, 10_000_000),
            Point::new(0, 10_000_000),
        ]),
        Vec::new(),
    );
    let lines = fill_honeycomb(&domain, config, scale).unwrap();
    // Standalone C++ CacheData calculation: distance=2713860,
    // x_offset=203539, y_offset=117513 (integer/float division).
    assert!(
        lines
            .iter()
            .any(|line| line.points().contains(&Point::new(2510321, 117513)))
    );
}

#[test]
fn direction_overrides_and_thickness_preserve_upstream_layer_phase() {
    let scale = CoordinateScale::Normal;
    let domain = surface(scale);
    let first = fill_honeycomb(&domain, params(0), scale).unwrap();
    for layer in 0..6 {
        let mut config = params(layer);
        config.fixed_angle = true;
        assert_eq!(fill_honeycomb(&domain, config, scale).unwrap(), first);
        config.fixed_angle = false;
        config.angle = 0.8;
        config.bridge_angle = Some(0.0);
        assert_eq!(fill_honeycomb(&domain, config, scale).unwrap(), first);
        config = params(layer);
        config.thickness_layers = 2;
        assert_eq!(
            fill_honeycomb(&domain, config, scale).unwrap(),
            fill_honeycomb(&domain, params(layer / 2), scale).unwrap()
        );
    }
}

#[test]
fn simplification_retains_hexagon_corners() {
    for scale in [CoordinateScale::Normal, CoordinateScale::LargeBed] {
        let lines = fill_honeycomb(&surface(scale), params(0), scale).unwrap();
        assert!(!lines.is_empty());
        for segment in lines.iter().flat_map(|line| line.points().windows(2)) {
            let dx = scale.unscale(segment[1].x() - segment[0].x());
            let dy = scale.unscale(segment[1].y() - segment[0].y());
            assert!(
                (dx * dx + dy * dy).sqrt() < 4.0,
                "a 3 mm-pitch hexagon must not become a multi-cell chord: {segment:?}"
            );
        }
    }
}
