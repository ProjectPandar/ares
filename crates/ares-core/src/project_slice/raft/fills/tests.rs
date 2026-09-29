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
        density: 0.9,
        sheath: false,
        width_mm: 0.42,
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
