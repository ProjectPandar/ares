mod anchors;
mod errors;
mod geometry;

use super::{DETECT_BRIDGE_DIRECTIONS, helpers::*};
use crate::{
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, Line, Point, Polygon, Polyline,
        WaveSeed, detect_bridging_direction, difference_open_polylines, offset_paths,
    },
    project_slice::prepare_infill::external_surfaces::{Bridge, ExpansionZone},
};

type DetectBridgeDirectionsFn =
    fn(&[WaveSeed], &mut [Bridge], &[ExpansionZone], CoordinateScale) -> Result<(), ClipperError>;

pub(super) const DETECT: DetectBridgeDirectionsFn = DETECT_BRIDGE_DIRECTIONS;

pub(super) struct Manual {
    pub(super) expanded: Vec<Polygon>,
    pub(super) fragments: Vec<Polyline>,
    pub(super) lines: Vec<Line>,
    pub(super) direction: (f64, f64),
    pub(super) cost: f64,
    pub(super) angle: f64,
}

pub(super) fn bridge(expolygon: ExPolygon, angle: Option<f64>) -> Bridge {
    Bridge {
        expolygon,
        group_id: 0,
        bridge_expansion_begin: 0,
        angle,
    }
}

pub(super) fn seed(src: u32, boundary: u32, path: &[(i64, i64)]) -> WaveSeed {
    WaveSeed {
        src,
        boundary,
        path: polygon(path),
    }
}

pub(super) fn rectangle(min_x: i64, min_y: i64, max_x: i64, max_y: i64) -> ExPolygon {
    expolygon(
        &[
            (min_x, min_y),
            (max_x, min_y),
            (max_x, max_y),
            (min_x, max_y),
        ],
        Vec::new(),
    )
}

pub(super) fn bridge_polygons(expolygon: &ExPolygon) -> Vec<Polygon> {
    std::iter::once(expolygon.contour())
        .chain(expolygon.holes())
        .cloned()
        .collect()
}

pub(super) fn manual(
    expolygon: &ExPolygon,
    anchor_areas: &[Polygon],
    scale: CoordinateScale,
) -> Result<Manual, ClipperError> {
    let overhang = bridge_polygons(expolygon);
    let paths = overhang
        .iter()
        .map(Polygon::split_at_first_point)
        .collect::<Vec<_>>();
    let epsilon = (1e-4_f64 / scale.factor()) as f32;
    assert!(epsilon > 0.0);
    let expanded = offset_paths(anchor_areas, epsilon, JoinType::Miter, 3.0)?;
    let fragments = difference_open_polylines(&paths, &expanded)?;
    let lines = fragments
        .iter()
        .flat_map(|path| {
            path.points()
                .windows(2)
                .map(|pair| Line::new(pair[0], pair[1]))
        })
        .collect::<Vec<_>>();
    let (direction, cost) = detect_bridging_direction(&lines, &overhang, scale);
    let angle = std::f64::consts::PI + direction.1.atan2(direction.0);
    Ok(Manual {
        expanded,
        fragments,
        lines,
        direction,
        cost,
        angle,
    })
}

pub(super) fn point_pairs(path: &Polyline) -> Vec<(i64, i64)> {
    path.points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}

pub(super) fn fragment_points(paths: &[Polyline]) -> Vec<Vec<(i64, i64)>> {
    paths.iter().map(point_pairs).collect()
}

pub(super) fn line_points(lines: &[Line]) -> Vec<((i64, i64), (i64, i64))> {
    lines
        .iter()
        .map(|line| ((line.a.x(), line.a.y()), (line.b.x(), line.b.y())))
        .collect()
}

pub(super) fn polygon_points(paths: &[Polygon]) -> Vec<Vec<(i64, i64)>> {
    paths
        .iter()
        .map(|path| {
            path.points()
                .iter()
                .map(|point| (point.x(), point.y()))
                .collect()
        })
        .collect()
}

pub(super) fn angles(bridges: &[Bridge]) -> Vec<Option<u64>> {
    bridges
        .iter()
        .map(|bridge| bridge.angle.map(f64::to_bits))
        .collect()
}

pub(super) fn invalid_triangle() -> ExPolygon {
    const OUTSIDE: i64 = 0x4000_0000_0000_0000;
    expolygon(
        &[(OUTSIDE, 0), (OUTSIDE, 10), (OUTSIDE - 1, 10)],
        Vec::new(),
    )
}

const _: fn(i64, i64) -> Point = Point::new;
