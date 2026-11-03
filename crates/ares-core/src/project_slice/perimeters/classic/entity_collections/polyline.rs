//! Native ExtrusionEntity polyline contracts; shared with deferred loops.

use super::super::{chained_loops::ExtrusionLoop, materialize::ExtrusionPath};
use super::types::ExtrusionEntity;
use crate::geometry::Point;

impl ExtrusionEntity {
    pub(in crate::project_slice) fn as_polyline(&self) -> Vec<Point> {
        match self {
            ExtrusionEntity::Loop(loop_) => loop_polyline(&loop_.extrusion_loop),
            ExtrusionEntity::Path(path) => path
                .polyline
                .points
                .iter()
                .map(|point| Point::new(point.x, point.y))
                .collect(),
            ExtrusionEntity::MultiPath(multi) => multipath_polyline(&multi.paths),
        }
    }
}

fn loop_polyline(loop_: &ExtrusionLoop) -> Vec<Point> {
    // `ExtrusionLoop::as_polyline/polygon`: omit each path's last point,
    // then close the polygon at its first point. Do not flatten join duplicates.
    if !loop_.paths.iter().any(|path| {
        path.polyline
            .points
            .windows(2)
            .any(|pair| pair[0] != pair[1])
    }) {
        return Vec::new();
    }
    let mut points: Vec<_> = loop_
        .paths
        .iter()
        .flat_map(|path| &path.polyline.points[..path.polyline.points.len() - 1])
        .map(|point| Point::new(point.x, point.y))
        .collect();
    points.push(points[0]);
    points
}

fn multipath_polyline(paths: &[ExtrusionPath]) -> Vec<Point> {
    let Some(first) = paths.first() else {
        return Vec::new();
    };
    let mut points = vec![Point::new(
        first.polyline.points[0].x,
        first.polyline.points[0].y,
    )];
    points.extend(paths.iter().flat_map(|path| {
        path.polyline
            .points
            .iter()
            .skip(1)
            .map(|point| Point::new(point.x, point.y))
    }));
    points
}
