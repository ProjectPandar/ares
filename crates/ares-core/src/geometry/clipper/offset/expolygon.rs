// ClipperUtils.cpp:c2_offset_expolygons: normalize only declared contour/hole roles.
use super::*;
use crate::geometry::ExPolygon;

fn normalized<'a, T: Output>(
    polygons: impl IntoIterator<Item = &'a ExPolygon>,
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> T {
    let mut paths = Vec::new();
    let shortest = (f64::from(delta) * 0.005).abs();
    for polygon in polygons {
        if !append_path(
            &mut paths,
            polygon.contour().points(),
            shortest,
            c2::EndType::Polygon,
        ) {
            continue;
        }
        let area = c2::area(paths.last().unwrap());
        if area == 0.0 {
            if delta < 0.0 {
                paths.pop();
            }
            continue;
        } else if area < 0.0 {
            paths.last_mut().unwrap().reverse();
        }
        for hole in polygon.holes() {
            append_offset_hole(&mut paths, hole, shortest);
        }
    }
    let mut co = setup(delta, kind, limit, 0.0);
    co.add_paths(&paths, join(kind), c2::EndType::Polygon);
    let mut output = T::default();
    output.offset(&mut co, f64::from(delta));
    output
}

// ClipperUtils.cpp:c2_offset_expolygons, retained-hole orientation block.
fn append_offset_hole(paths: &mut c2::Paths64, hole: &Polygon, shortest: f64) {
    if !append_path(paths, hole.points(), shortest, c2::EndType::Polygon) {
        return;
    }
    let area = c2::area(paths.last().unwrap());
    if area == 0.0 {
        paths.pop();
    } else if area > 0.0 {
        paths.last_mut().unwrap().reverse();
    }
}

pub(crate) fn offset_expolygon(
    polygon: &ExPolygon,
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let tree: c2::PolyTree64 = normalized(std::iter::once(polygon), delta, kind, limit);
    Ok(PolyTree::new(tree).into_expolygons(crate::geometry::CoordinateScale::Normal))
}
pub(crate) fn offset_expolygons(
    polygons: &[ExPolygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let tree: c2::PolyTree64 = normalized(polygons, delta, kind, limit);
    Ok(PolyTree::new(tree).into_expolygons(crate::geometry::CoordinateScale::Normal))
}
pub(crate) fn offset_expolygons_paths(
    polygons: &[ExPolygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let paths: c2::Paths64 = normalized(polygons, delta, kind, limit);
    Ok(backend::polygons(&paths))
}
pub(crate) fn offset_expolygon_refs_paths(
    polygons: &[&ExPolygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let paths: c2::Paths64 = normalized(polygons.iter().copied(), delta, kind, limit);
    Ok(backend::polygons(&paths))
}
pub(crate) fn offset2_ex(
    polygons: &[ExPolygon],
    first: f32,
    second: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    offset2_ex_with_interstage(polygons, (first, second, kind, limit), || Ok(()))
}
pub(crate) fn offset2_ex_with_interstage(
    polygons: &[ExPolygon],
    (first, second, kind, limit): (f32, f32, JoinType, f64),
    interstage: impl FnOnce() -> Result<(), ClipperError>,
) -> Result<Vec<ExPolygon>, ClipperError> {
    let first: c2::Paths64 = normalized(polygons, first, kind, limit);
    interstage()?;
    let mut paths = Vec::with_capacity(first.len());
    for path in &first {
        append_path(
            &mut paths,
            path,
            (f64::from(second) * 0.005).abs(),
            c2::EndType::Polygon,
        );
    }
    let tree: c2::PolyTree64 = closed(paths, second, kind, limit);
    Ok(PolyTree::new(tree).into_expolygons(crate::geometry::CoordinateScale::Normal))
}
pub(crate) fn opening_ex(
    polygons: &[ExPolygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    offset2_ex(polygons, -delta, delta, kind, limit)
}
pub(crate) fn closing_ex(
    polygons: &[ExPolygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<ExPolygon>, ClipperError> {
    offset2_ex(polygons, delta, -delta, kind, limit)
}
