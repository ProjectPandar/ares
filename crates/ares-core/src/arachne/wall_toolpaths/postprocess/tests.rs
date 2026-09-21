use crate::{
    arachne::{ExtrusionJunction, ExtrusionLine},
    geometry::{CoordinateScale, Point},
};

use super::{
    RawWallToolPathConfig, remove_small_lines, separate_inner_contour, simplify_toolpaths,
};

fn simplify_config() -> RawWallToolPathConfig {
    let scale = CoordinateScale::Normal;
    RawWallToolPathConfig {
        outer_spacing: 0,
        inner_spacing: 0,
        inset_count: 1,
        outer_wall_inset: 0,
        layer_height: 0,
        min_bead_width: 0,
        min_feature_size: 0,
        transition_length: 0,
        transitioning_angle: 0.0,
        transition_filter_deviation: 0,
        wall_distribution_count: 1,
        min_length_factor: 0.5,
        wall_maximum_resolution: scale.checked_scale(0.5).unwrap(),
        wall_maximum_deviation: scale.checked_scale(0.025).unwrap(),
        is_top_or_bottom_layer: false,
        coordinate_scale: scale,
    }
}

fn odd_line(length: i64) -> ExtrusionLine {
    let mut line = ExtrusionLine::new(0, true);
    line.push(ExtrusionJunction::new(Point::new(0, 0), 100, 0));
    line.push(ExtrusionJunction::new(Point::new(length, 0), 100, 0));
    line
}

#[test]
fn task22o197_removes_only_odd_open_line_below_width_factor() {
    let mut toolpaths = vec![vec![odd_line(20), odd_line(30)]];

    remove_small_lines(&mut toolpaths, 0.5, false);

    assert_eq!(toolpaths[0], vec![odd_line(30)]);
}

#[test]
fn task22o198_separates_closed_even_zero_width_inner_contour() {
    let mut contour = ExtrusionLine::new(1, false);
    contour.is_closed = true;
    for point in [
        Point::new(0, 0),
        Point::new(100, 0),
        Point::new(100, 100),
        Point::new(0, 100),
    ] {
        contour.push(ExtrusionJunction::new(point, 0, 1));
    }
    let mut ignored_odd = odd_line(20);
    for junction in &mut ignored_odd.junctions {
        junction.width = 0;
    }
    let printable = odd_line(30);
    let mut toolpaths = vec![vec![contour], vec![ignored_odd], vec![printable.clone()]];

    let inner_contour = separate_inner_contour(&mut toolpaths);

    assert_eq!(toolpaths, vec![vec![printable]]);
    assert_eq!(inner_contour.len(), 1);
    assert_eq!(inner_contour[0].points().len(), 4);
}

#[test]
fn task22o199_simplifies_variable_width_lines_with_source_defaults() {
    let mut line = ExtrusionLine::new(0, false);
    for x in [0, 100_000, 200_000] {
        line.push(ExtrusionJunction::new(Point::new(x, 0), 400_000, 0));
    }
    let mut toolpaths = vec![vec![line]];

    simplify_toolpaths(&mut toolpaths, simplify_config());

    assert_eq!(toolpaths[0][0].junctions.len(), 2);
    assert_eq!(toolpaths[0][0].junctions[0].point, Point::new(0, 0));
    assert_eq!(toolpaths[0][0].junctions[1].point, Point::new(200_000, 0));
}
