mod branches_errors;
mod oracle;
mod ownership;

use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, Point, Polygon, RegionExpansion,
    merge_expansions_into_expolygons,
};

type PathSnapshot = Vec<(i64, i64)>;
type ExPolygonSnapshot = (PathSnapshot, Vec<PathSnapshot>);

type MergeResult = Result<Vec<ExPolygon>, ClipperError>;
type MergeFn = fn(Vec<ExPolygon>, Vec<RegionExpansion>, CoordinateScale) -> MergeResult;

const MERGE: MergeFn = merge_expansions_into_expolygons;

fn polygon(points: &[(i64, i64)]) -> Polygon {
    Polygon::new(points.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

fn expolygon(contour: &[(i64, i64)], holes: Vec<Polygon>) -> ExPolygon {
    ExPolygon::new(polygon(contour), holes)
}

fn rectangle(left: i64, bottom: i64, right: i64, top: i64) -> Polygon {
    polygon(&[(left, bottom), (right, bottom), (right, top), (left, top)])
}

fn square(left: i64, right: i64) -> ExPolygon {
    ExPolygon::new(rectangle(left, 0, right, 100), Vec::new())
}

fn expansion(src_id: u32, boundary_id: u32, points: &[(i64, i64)]) -> RegionExpansion {
    RegionExpansion {
        polygon: polygon(points),
        src_id,
        boundary_id,
    }
}

fn snapshot(expolygons: &[ExPolygon]) -> Vec<ExPolygonSnapshot> {
    expolygons
        .iter()
        .map(|expolygon| {
            (
                path_snapshot(expolygon.contour()),
                expolygon.holes().iter().map(path_snapshot).collect(),
            )
        })
        .collect()
}

fn path_snapshot(polygon: &Polygon) -> PathSnapshot {
    polygon
        .points()
        .iter()
        .map(|point| (point.x(), point.y()))
        .collect()
}
