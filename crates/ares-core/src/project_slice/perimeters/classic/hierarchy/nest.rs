use super::types::{LoopBuckets, PerimeterGeneratorLoop};
use crate::geometry::Point;

pub(super) fn nest(mut buckets: LoopBuckets) -> NestedLoops {
    let count = buckets.contours.len();
    if count == 0 {
        return NestedLoops {
            roots: Vec::new(),
            contours: buckets.contours,
            holes: buckets.holes,
        };
    }

    let bounds = BucketBounds {
        contours: collect_bounds(&buckets.contours),
        holes: collect_bounds(&buckets.holes),
    };

    for depth in 0..count {
        let candidates = std::mem::take(&mut buckets.holes[depth]);
        let mut kept = Vec::new();
        for child in candidates {
            let first = child.polygon.points()[0];
            if let Some(parent) = find_hole_parent(&buckets, &bounds, depth, first) {
                append_child(&mut buckets, parent, child);
            } else {
                kept.push(child);
            }
        }
        buckets.holes[depth] = kept;
    }

    for depth in (1..count).rev() {
        let candidates = std::mem::take(&mut buckets.contours[depth]);
        let mut kept = Vec::new();
        for child in candidates {
            let first = child.polygon.points()[0];
            if let Some(parent) = find_contour_parent(&buckets, &bounds, depth, first) {
                append_child(&mut buckets, parent, child);
            } else {
                kept.push(child);
            }
        }
        buckets.contours[depth] = kept;
    }

    let roots = std::mem::take(&mut buckets.contours[0]);
    NestedLoops {
        roots,
        contours: buckets.contours,
        holes: buckets.holes,
    }
}

struct BucketBounds {
    contours: Vec<Vec<[i64; 4]>>,
    holes: Vec<Vec<[i64; 4]>>,
}

fn collect_bounds(levels: &[Vec<PerimeterGeneratorLoop>]) -> Vec<Vec<[i64; 4]>> {
    levels
        .iter()
        .map(|level| {
            level
                .iter()
                .map(|item| polygon_bounds(item.polygon.points()))
                .collect()
        })
        .collect()
}

fn polygon_bounds(points: &[Point]) -> [i64; 4] {
    let Some(&first) = points.first() else {
        return [i64::MAX, i64::MAX, i64::MIN, i64::MIN];
    };
    let mut bounds = [first.x(), first.y(), first.x(), first.y()];
    for point in &points[1..] {
        bounds[0] = bounds[0].min(point.x());
        bounds[1] = bounds[1].min(point.y());
        bounds[2] = bounds[2].max(point.x());
        bounds[3] = bounds[3].max(point.y());
    }
    bounds
}

fn contains_with_bounds(
    loops: &[PerimeterGeneratorLoop],
    bounds: &[[i64; 4]],
    point: Point,
) -> Option<usize> {
    loops.iter().zip(bounds).position(|(candidate, bbox)| {
        point.x() >= bbox[0]
            && point.x() <= bbox[2]
            && point.y() >= bbox[1]
            && point.y() <= bbox[3]
            && candidate.polygon.contains(&point)
    })
}

fn find_hole_parent(
    buckets: &LoopBuckets,
    bounds: &BucketBounds,
    depth: usize,
    first: Point,
) -> Option<Parent> {
    for parent_depth in (depth + 1)..buckets.holes.len() {
        if let Some(index) = contains_with_bounds(
            &buckets.holes[parent_depth],
            &bounds.holes[parent_depth],
            first,
        ) {
            return Some(Parent::Hole(parent_depth, index));
        }
    }
    for parent_depth in (0..buckets.contours.len()).rev() {
        if let Some(index) = contains_with_bounds(
            &buckets.contours[parent_depth],
            &bounds.contours[parent_depth],
            first,
        ) {
            return Some(Parent::Contour(parent_depth, index));
        }
    }
    None
}

fn find_contour_parent(
    buckets: &LoopBuckets,
    bounds: &BucketBounds,
    depth: usize,
    first: Point,
) -> Option<Parent> {
    for parent_depth in (0..depth).rev() {
        if let Some(index) = contains_with_bounds(
            &buckets.contours[parent_depth],
            &bounds.contours[parent_depth],
            first,
        ) {
            return Some(Parent::Contour(parent_depth, index));
        }
    }
    None
}

fn append_child(buckets: &mut LoopBuckets, parent: Parent, child: PerimeterGeneratorLoop) {
    match parent {
        Parent::Hole(depth, index) => buckets.holes[depth][index].children.push(child),
        Parent::Contour(depth, index) => buckets.contours[depth][index].children.push(child),
    }
}

#[derive(Clone, Copy)]
enum Parent {
    Hole(usize, usize),
    Contour(usize, usize),
}

pub(super) struct NestedLoops {
    pub(super) roots: Vec<PerimeterGeneratorLoop>,
    pub(super) contours: Vec<Vec<PerimeterGeneratorLoop>>,
    pub(super) holes: Vec<Vec<PerimeterGeneratorLoop>>,
}
