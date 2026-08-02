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

    for depth in 0..count {
        let mut index = 0;
        while index < buckets.holes[depth].len() {
            let first = buckets.holes[depth][index].polygon.points()[0];
            let parent = find_hole_parent(&buckets, depth, first);
            if let Some(parent) = parent {
                let child = buckets.holes[depth].remove(index);
                append_child(&mut buckets, parent, child);
            } else {
                index += 1;
            }
        }
    }

    for depth in (1..count).rev() {
        let mut index = 0;
        while index < buckets.contours[depth].len() {
            let first = buckets.contours[depth][index].polygon.points()[0];
            let parent = find_contour_parent(&buckets, depth, first);
            if let Some(parent) = parent {
                let child = buckets.contours[depth].remove(index);
                append_child(&mut buckets, parent, child);
            } else {
                index += 1;
            }
        }
    }

    let roots = std::mem::take(&mut buckets.contours[0]);
    NestedLoops {
        roots,
        contours: buckets.contours,
        holes: buckets.holes,
    }
}

fn find_hole_parent(buckets: &LoopBuckets, depth: usize, first: Point) -> Option<Parent> {
    for parent_depth in (depth + 1)..buckets.holes.len() {
        if let Some(index) = buckets.holes[parent_depth]
            .iter()
            .position(|candidate| candidate.polygon.contains(&first))
        {
            return Some(Parent::Hole(parent_depth, index));
        }
    }
    for parent_depth in (0..buckets.contours.len()).rev() {
        if let Some(index) = buckets.contours[parent_depth]
            .iter()
            .position(|candidate| candidate.polygon.contains(&first))
        {
            return Some(Parent::Contour(parent_depth, index));
        }
    }
    None
}

fn find_contour_parent(buckets: &LoopBuckets, depth: usize, first: Point) -> Option<Parent> {
    for parent_depth in (0..depth).rev() {
        if let Some(index) = buckets.contours[parent_depth]
            .iter()
            .position(|candidate| candidate.polygon.contains(&first))
        {
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

#[cfg(test)]
mod tests;
