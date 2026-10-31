// ClipperUtils.cpp:c2_attach_stray_holes/path_encloses/interior_point.
use super::{backend, c2};
use crate::geometry::{CoordinateScale, ExPolygon, Point, Polygon, fixed_gcc_sort_by};

pub(super) fn attach(
    tree: &c2::PolyTree64,
    output: &mut Vec<ExPolygon>,
    stray: &[usize],
    scale: CoordinateScale,
) {
    if stray.is_empty() {
        return;
    }
    let bounds: Vec<_> = output
        .iter()
        .map(|p| c2::get_bounds_path(&backend::to_path(p.contour().points())))
        .collect();
    let index = Index::new(&bounds, scale);
    let mut areas = vec![0.0; output.len()];
    let mut candidates = Vec::new();
    for &node in stray {
        let path = tree.nodes[node].polygon();
        let hole = backend::polygon(path);
        let hole_area = -c2::area(path);
        let bounds = c2::get_bounds_path(path);
        candidates.clear();
        index.candidates(bounds, &mut candidates);
        for &candidate in &candidates {
            if areas[candidate] == 0.0 {
                areas[candidate] = output[candidate].contour().area();
            }
        }
        fixed_gcc_sort_by(&mut candidates, |&a, &b| areas[a] < areas[b]);
        let first = candidates.partition_point(|&candidate| areas[candidate] <= hole_area);
        if let Some(&parent) = candidates[first..]
            .iter()
            .find(|&&i| encloses(output[i].contour(), &hole))
        {
            output[parent].holes_mut().push(hole);
        } else {
            output.push(ExPolygon::new(hole, Vec::new()));
        }
    }
}

pub(super) fn encloses(outer: &Polygon, inner: &Polygon) -> bool {
    for &point in inner.points() {
        if !outer.on_boundary(point, 3.0) {
            return outer.contains(&point);
        }
    }
    interior_point(inner).is_some_and(|point| outer.contains(&point))
}

// ClipperUtils.cpp:935–971. Band and crossing tie behavior stays with std::sort.
fn interior_point(polygon: &Polygon) -> Option<Point> {
    let points = polygon.points();
    let mut heights: Vec<_> = points.iter().map(|p| p.y()).collect();
    heights.sort_unstable();
    heights.dedup();
    let mut bands: Vec<_> = heights
        .windows(2)
        .filter(|p| p[1] - p[0] >= 2)
        .map(|p| (p[0], p[1]))
        .collect();
    fixed_gcc_sort_by(&mut bands, |a, b| a.1 - a.0 > b.1 - b.0);
    let mut crossings = Vec::new();
    for (low, high) in bands {
        let y = low + (high - low) / 2;
        crossings.clear();
        let mut previous = points.len() - 1;
        for i in 0..points.len() {
            let (a, b) = (points[previous], points[i]);
            if (a.y() > y) != (b.y() > y) {
                crossings.push(
                    a.x() as f64
                        + (y - a.y()) as f64 * (b.x() - a.x()) as f64 / (b.y() - a.y()) as f64,
                );
            }
            previous = i;
        }
        fixed_gcc_sort_by(&mut crossings, |a, b| a < b);
        let (mut left, mut right) = (0.0, 0.0);
        for i in (1..crossings.len()).step_by(2) {
            if crossings[i] - crossings[i - 1] > right - left {
                left = crossings[i - 1];
                right = crossings[i];
            }
        }
        let point = Point::new((0.5_f64 * (left + right)).round() as i64, y);
        if right > left && super::super::point_in_polygon(point, points) == 1 {
            return Some(point);
        }
    }
    None
}

mod bbox_index;
use bbox_index::Index;
