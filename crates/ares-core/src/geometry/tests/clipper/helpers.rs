use crate::geometry::clipper::ordering::{SortTrace, fixed_msvc_sort_by_for_test};
use crate::geometry::clipper::{ClipOperation, Clipper, ClipperOptions, FillRule, PathRole};
use crate::geometry::{Point, Polygon};

#[derive(Clone, Copy, Debug)]
struct SortItem {
    identity: usize,
    key: i64,
}

pub(super) fn point(x: i64, y: i64) -> Point {
    Point::new(x, y)
}

pub(super) fn polygon(coordinates: &[(i64, i64)]) -> Polygon {
    Polygon::new(coordinates.iter().map(|&(x, y)| Point::new(x, y)).collect())
}

pub(super) fn square() -> Polygon {
    polygon(&[(0, 0), (10, 0), (10, 10), (0, 10)])
}

pub(super) fn polygons(coordinates: &[&[(i64, i64)]]) -> Vec<Polygon> {
    coordinates.iter().map(|points| polygon(points)).collect()
}

pub(super) fn execute(
    subject: Vec<Polygon>,
    clip: Vec<Polygon>,
    operation: ClipOperation,
    fills: (FillRule, FillRule),
    options: ClipperOptions,
) -> Vec<Polygon> {
    let mut clipper = Clipper::new(options);
    clipper
        .add_closed_paths(&subject, PathRole::Subject)
        .expect("oracle subject coordinates are in range");
    clipper
        .add_closed_paths(&clip, PathRole::Clip)
        .expect("oracle clip coordinates are in range");
    clipper
        .execute_paths(operation, fills.0, fills.1)
        .expect("closed Clipper execution accepts flat output")
}

pub(super) fn traced_fixed_sort(keys: &[i64], descending: bool) -> (Vec<usize>, SortTrace) {
    let mut items: Vec<_> = keys
        .iter()
        .enumerate()
        .map(|(identity, &key)| SortItem { identity, key })
        .collect();
    let trace = fixed_msvc_sort_by_for_test(
        &mut items,
        |left, right| {
            if descending {
                left.key > right.key
            } else {
                left.key < right.key
            }
        },
        |item| item.identity,
    );

    (items.into_iter().map(|item| item.identity).collect(), trace)
}
