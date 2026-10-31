use super::*;

pub(crate) fn offset_paths(
    paths: &[Polygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let output: c2::Paths64 = closed(
        prepare(paths, delta, c2::EndType::Polygon),
        delta,
        kind,
        limit,
    );
    Ok(backend::polygons(&output))
}
pub(crate) fn offset_paths_tree(
    paths: &[Polygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<PolyTree, ClipperError> {
    Ok(PolyTree::new(closed(
        prepare(paths, delta, c2::EndType::Polygon),
        delta,
        kind,
        limit,
    )))
}
// ClipperUtils.cpp:582–598: the single-polygon overload preserves CW orientation.
pub(crate) fn raw_offset_paths(
    paths: &[Polygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    let mut output = Vec::new();
    for polygon in paths {
        let mut paths = Vec::new();
        if !append_path(
            &mut paths,
            polygon.points(),
            (f64::from(delta) * 0.005).abs(),
            c2::EndType::Polygon,
        ) {
            continue;
        }
        let area = c2::area(&paths[0]);
        if area == 0.0 && delta < 0.0 {
            continue;
        }
        let mut co = setup(delta, kind, limit, 0.0);
        co.add_paths(&paths, join(kind), c2::EndType::Polygon);
        let mut single = Vec::new();
        co.execute(
            f64::from(if area < 0.0 { -delta } else { delta }),
            &mut single,
        );
        output.extend(backend::polygons(&single));
    }
    Ok(output)
}
pub(crate) fn offset_open_paths(
    paths: &[Polygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
) -> Result<Vec<Polygon>, ClipperError> {
    offset_lines(paths, delta, kind, limit, (c2::EndType::Butt, 0.0))
}
// ClipperUtils.cpp:c2_offset_lines, including round cap tolerance.
pub(crate) fn offset_lines(
    paths: &[Polygon],
    delta: f32,
    kind: JoinType,
    limit: f64,
    (end, cap): (c2::EndType, f64),
) -> Result<Vec<Polygon>, ClipperError> {
    if end == c2::EndType::Polygon {
        return offset_paths(paths, delta, kind, limit);
    }
    let paths = prepare(paths, delta, end);
    let mut co = setup(delta, kind, limit, cap);
    co.add_paths(&paths, join(kind), end);
    let mut output = Vec::new();
    co.execute(f64::from(delta), &mut output);
    Ok(backend::polygons(&output))
}
