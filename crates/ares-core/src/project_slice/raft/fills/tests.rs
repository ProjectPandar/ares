use super::*;
use crate::geometry::Polygon;

#[test]
fn support_fill_preserves_the_oracle_wipe_corner() {
    let polygon = Polygon::new(
        [
            (7_650_587, -7_357_679),
            (7_650_587, 7_357_694),
            (7_357_694, 7_650_587),
            (-7_357_679, 7_650_587),
            (-7_650_572, 7_357_694),
            (-7_650_572, -7_357_679),
            (-7_357_679, -7_650_572),
            (7_357_694, -7_650_572),
        ]
        .into_iter()
        .map(|(x, y)| Point::new(x, y))
        .collect(),
    );
    let mut fill = spec(RaftFillPattern::SupportBase, 0.627_615_75);
    fill.angle = f64::from(3.0 * std::f32::consts::FRAC_PI_4);
    fill.spacing = 0.337_079_644_203_186_04;
    let paths =
        raft_layer_fill(&[polygon], fill, Point::new(0, 0), CoordinateScale::Normal).unwrap();
    // Independently observed Orca endpoint Y102.712134 with origin (110,110).
    // Double trig instead moves this corner one lattice unit down; XY output
    // still rounds to the same 3 decimals, but wipe extrusion does not.
    assert!(
        paths
            .iter()
            .any(|path| path.points().contains(&Point::new(-7_482_033, -7_287_866))),
        "missing reference wipe corner"
    );
}

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
        no_sort: false,
        tree_double_wall_area: 0.0,
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

pub(super) fn square(min: i64, max: i64) -> Polygon {
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
        no_sort: false,
        tree_double_wall_area: 0.0,
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

pub(super) const MM: i64 = 1_000_000;

pub(super) fn sheath_spec(sheath_spacing: f64) -> RaftFillSpec {
    RaftFillSpec {
        sheath: true,
        sheath_spacing,
        ..spec(RaftFillPattern::SupportBase, 0.2)
    }
}

fn signed_area(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| {
            (pair[0].x() as f64) * (pair[1].y() as f64)
                - (pair[1].x() as f64) * (pair[0].y() as f64)
        })
        .sum::<f64>()
        / 2.0
}

/// The sheath clip (`0.15·spacing`) and the fill inset (`-0.4·spacing`)
/// use the layer's own flow spacing (`SupportCommon.cpp:733-749`), not the
/// filler spacing.
#[test]
fn sheath_offsets_use_the_layer_flow_spacing() {
    let out = sheath_polylines(
        &[square(0, 20 * MM)],
        sheath_spec(2.0),
        CoordinateScale::Normal,
    )
    .unwrap();
    let sheath = out[0].points();
    let (first, last) = (sheath[0], sheath[sheath.len() - 1]);
    let gap = ((first.x() - last.x()) as f64).hypot((first.y() - last.y()) as f64);
    assert!((gap - 0.3 * MM as f64).abs() < 1_000.0, "clip gap {gap}");
    // Sheath at `0.5·width` inside, fill region a further `0.4·2mm`.
    let fill_min = out[1..]
        .iter()
        .flat_map(|polyline| polyline.points())
        .map(|point| point.x().min(point.y()))
        .min()
        .unwrap();
    assert!(fill_min >= MM + 10 * 1_000 - 1_000, "fill min {fill_min}");
}

/// `closing_ex(const Polygons&)` keeps clockwise hole polygons as holes;
/// `draw_perimeters` draws each hole reversed (`SupportCommon.cpp:
/// 464-472`) and the filler gets the whole `ExPolygon` (`:749`).
#[test]
fn sheath_draws_hole_loops_and_fills_around_the_hole() {
    let mut hole = square(8 * MM, 12 * MM);
    hole.reverse();
    let out = sheath_polylines(
        &[square(0, 20 * MM), hole],
        sheath_spec(0.4),
        CoordinateScale::Normal,
    )
    .unwrap();
    let hole_loop = out[1].points();
    assert!(hole_loop.iter().all(|point| {
        (7 * MM..=13 * MM).contains(&point.x()) && (7 * MM..=13 * MM).contains(&point.y())
    }));
    assert!(
        signed_area(hole_loop) > 0.0,
        "hole loop runs CCW after the reversal"
    );
    let inside_hole = |point: &Point| {
        let inner = 7 * MM + 950_000..13 * MM - 950_000;
        inner.contains(&point.x()) && inner.contains(&point.y())
    };
    assert!(
        out[2..]
            .iter()
            .flat_map(|polyline| polyline.points())
            .all(|point| !inside_hole(point))
    );
}

fn sheath_polylines(
    polygons: &[Polygon],
    spec: RaftFillSpec,
    scale: CoordinateScale,
) -> Result<Vec<Polyline>, ClipperError> {
    Ok(raft_layer_entities(polygons, spec, scale)?
        .iter()
        .flat_map(RaftEntity::polylines)
        .cloned()
        .collect())
}

/// The flange passes `no_sort = true` (`SupportCommon.cpp:1517`): each
/// region is one collection, its sheath loop first (`:741-750`).
#[test]
fn no_sort_sheath_keeps_one_collection_per_region() {
    let spec = RaftFillSpec {
        no_sort: true,
        ..sheath_spec(0.4)
    };
    let out = raft_layer_entities(
        &[square(0, 10 * MM), square(20 * MM, 30 * MM)],
        spec,
        CoordinateScale::Normal,
    )
    .unwrap();
    assert_eq!(out.len(), 2);
    for entity in &out {
        let RaftEntity::NoSort(polylines) = entity else {
            panic!("expected no_sort collections, got {out:?}");
        };
        assert!(polylines.len() > 1);
        let sheath = polylines[0].points();
        let (min_x, max_x) = sheath.iter().fold((i64::MAX, i64::MIN), |(lo, hi), point| {
            (lo.min(point.x()), hi.max(point.x()))
        });
        assert!((max_x - min_x - (10 * MM - 420_000)).abs() < 2_000);
    }
}
