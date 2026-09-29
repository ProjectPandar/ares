use super::*;
use crate::geometry::Polygon;

/// `FillSupportBase::fill_surface` calls `make_fill_lines` with
/// `x_margin = 0` (`FillRectilinear.cpp:3621`), so hatch lines within one
/// line width of the bounding-box edge are kept.
#[test]
fn support_base_keeps_hatch_lines_next_to_the_edge() {
    let (min, max) = (101_059_000, 118_941_000);
    let polygon = Polygon::new(vec![
        Point::new(min, min),
        Point::new(max, min),
        Point::new(max, max),
        Point::new(min, max),
    ]);
    let spec = RaftFillSpec {
        angle: std::f64::consts::FRAC_PI_2,
        spacing: 0.407_086_4,
        sheath_spacing: 0.407_086_4,
        density: 0.9,
        sheath: false,
        width_mm: 0.42,
        pattern: RaftFillPattern::SupportBase,
        preprocess: RaftFillPreprocess::Closing,
    };
    let out = raft_layer_fill(&[polygon], spec, Point::new(0, 0), CoordinateScale::Normal).unwrap();
    // The fill boundary is the source inset by half a spacing; the
    // connector traces it, so only a hatch line strictly inside it counts.
    let (inset, margin) = (203_543, 407_086);
    // Hatch lines run along y here; the back-rotation leaves a 1nm wobble.
    let edge_hatch = out
        .iter()
        .flat_map(|polyline| polyline.points().windows(2))
        .any(|pair| {
            let (a, b) = (pair[0], pair[1]);
            (a.x() - b.x()).abs() <= 1
                && (a.y() - b.y()).abs() > 15_000_000
                && ((min + inset < a.x() && a.x() < min + margin)
                    || (max - margin < a.x() && a.x() < max - inset))
        });
    assert!(edge_hatch);
}

fn square(min: i64, max: i64) -> Polygon {
    Polygon::new(vec![
        Point::new(min, min),
        Point::new(max, min),
        Point::new(max, max),
        Point::new(min, max),
    ])
}

fn spec(pattern: RaftFillPattern, density: f32) -> RaftFillSpec {
    RaftFillSpec {
        angle: 0.0,
        spacing: 0.4,
        sheath_spacing: 0.4,
        density,
        sheath: false,
        width_mm: 0.42,
        pattern,
        preprocess: RaftFillPreprocess::Closing,
    }
}

/// A dense raft interface uses `FillRectilinear` (`SupportParameters.hpp:
/// 135`), whose `connect_infill` joins every hatch line into one zig-zag
/// through the boundary.
#[test]
fn rectilinear_pattern_connects_the_hatch_into_one_zig_zag() {
    let out = raft_layer_fill(
        &[square(0, 10_000_000)],
        spec(RaftFillPattern::Rectilinear, 1.0),
        Point::new(0, 0),
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(out.len(), 1);
    assert!(out[0].points().len() > 40);
}

/// A honeycomb `support_base_pattern` selects `FillHoneycomb`
/// (`SupportParameters.hpp:131-133`): its hexagon walls run diagonally.
#[test]
fn honeycomb_pattern_draws_diagonal_hexagon_walls() {
    let out = raft_layer_fill(
        &[square(0, 10_000_000)],
        spec(RaftFillPattern::Honeycomb, 0.2),
        Point::new(0, 0),
        CoordinateScale::Normal,
    )
    .unwrap();
    let diagonal = out
        .iter()
        .flat_map(|polyline| polyline.points().windows(2))
        .any(|pair| {
            let (dx, dy) = (pair[1].x() - pair[0].x(), pair[1].y() - pair[0].y());
            dx.abs() > 10_000 && dy.abs() > 10_000
        });
    assert!(diagonal);
}
