// `clip_extrusion` port from OrcaSlicer v2.4.2
// `PerimeterGenerator.cpp:311-357`: clips one open variable-width wall
// polyline (width as the Z coordinate) against closed polygons, keeping a
// width on every output vertex. The upstream ZFill callback interpolates the
// subject width at true edge crossings; this engine records those crossings
// as sentinel values instead, so callers must reject them (`z > 0` guard).

use super::{
    ClipOperation, Clipper, ClipperError, ClipperOffset, ClipperOptions, FillRule, JoinType,
    PathRole, Polygon,
    z::{KernelPoint, ZPath},
};
use crate::geometry::ExPolygon;

/// One clipped wall polyline vertex: `(x, y, width)` with the width in
/// scaled coordinates.
pub(crate) type ZWidthPath = Vec<(i64, i64, i64)>;

pub(crate) fn clip_open_path_widths(
    subject: &ZWidthPath,
    clip: &[Polygon],
    operation: ClipOperation,
) -> Result<Vec<ZWidthPath>, ClipperError> {
    let subject_path = subject
        .iter()
        .map(|&(x, y, width)| KernelPoint::new(x, y, width))
        .collect::<ZPath>();
    let mut clipper = Clipper::new(ClipperOptions::default());
    for polygon in clip {
        clipper.add_z_closed_path(
            &polygon
                .points()
                .iter()
                .map(|&point| KernelPoint::new(point.x(), point.y(), 0))
                .collect::<ZPath>(),
            PathRole::Clip,
        )?;
    }
    clipper.add_z_open_path(&subject_path, PathRole::Subject)?;
    let (paths, _) = clipper.execute_z_paths(operation, FillRule::NonZero, FillRule::NonZero);
    Ok(paths
        .into_iter()
        .map(|path| {
            path.into_iter()
                .map(|point| (point.x(), point.y(), point.z))
                .collect::<ZWidthPath>()
        })
        .collect())
}

/// A closed path whose vertices carry a source contour index as Z.
pub(crate) type ZTaggedPath = Vec<(i64, i64, i64)>;

/// `shrink_expolygon_with_contour_idx` (`SupportCommon.cpp:486-570`):
/// offsets `expolygon` inward by `delta`. Each output vertex is tagged with
/// its source contour index (0 = contour, 1 + i = hole i), or -1 where an
/// offset hole cuts the offset contour. Empty when nothing survives.
pub(crate) fn shrink_expolygon_with_contour_idx(
    expolygon: &ExPolygon,
    delta: f32,
    join_type: JoinType,
    miter_limit: f64,
) -> Result<Vec<ZTaggedPath>, ClipperError> {
    let offset = |polygon: &Polygon, delta: f32| {
        let mut co = ClipperOffset::default();
        if join_type == JoinType::Round {
            co.set_arc_tolerance(miter_limit);
        } else {
            co.set_miter_limit(miter_limit);
        }
        co.set_shortest_edge_length(f64::from(delta * 0.005));
        co.add_closed_path(polygon, join_type);
        co.execute_paths(f64::from(delta))
    };
    let tagged = |paths: Vec<Polygon>, z: i64| {
        paths.into_iter().map(move |path| {
            path.points()
                .iter()
                .map(|point| KernelPoint::new(point.x(), point.y(), z))
                .collect::<ZPath>()
        })
    };
    let contours_raw = offset(expolygon.contour(), -delta)?;
    if contours_raw.is_empty() {
        return Ok(Vec::new());
    }
    let contours = tagged(contours_raw, 0).collect::<Vec<_>>();
    // `Execute` reorients each CW hole to CCW, so `+delta` grows it.
    let mut holes = Vec::new();
    for (index, hole) in expolygon.holes().iter().enumerate() {
        holes.extend(tagged(offset(hole, delta)?, 1 + index as i64));
    }
    let output = if holes.is_empty() {
        contours
    } else {
        let mut clipper = Clipper::new(ClipperOptions::default());
        for path in &contours {
            clipper.add_z_closed_path(path, PathRole::Subject)?;
        }
        for path in &holes {
            clipper.add_z_closed_path(path, PathRole::Clip)?;
        }
        clipper.execute_z_paths_marked(
            ClipOperation::Difference,
            FillRule::NonZero,
            FillRule::NonZero,
            -1,
        )
    };
    Ok(output
        .into_iter()
        .map(|path| {
            path.into_iter()
                .map(|point| (point.x(), point.y(), point.z))
                .collect()
        })
        .collect())
}
