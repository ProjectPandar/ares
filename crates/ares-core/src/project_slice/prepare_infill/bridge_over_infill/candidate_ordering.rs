use std::cmp::Ordering;

use crate::geometry::{Polygon, fixed_msvc_sort_by};

use super::types::CandidateSurface;

#[derive(Clone, Copy, Default)]
struct SourceBounds {
    min_x: i64,
    min_y: i64,
    max_x: i64,
    max_y: i64,
    defined: bool,
}

pub(in crate::project_slice) fn order_candidate_surfaces(
    candidates: Vec<CandidateSurface>,
) -> Vec<CandidateSurface> {
    let bounds = candidates.iter().map(candidate_bounds).collect::<Vec<_>>();
    let mut order = (0..candidates.len()).collect::<Vec<_>>();
    fixed_msvc_sort_by(&mut order, |left, right| {
        let left = bounds[*left];
        let right = bounds[*right];
        left.min_x < right.min_x || (left.min_x == right.min_x && left.min_y < right.min_y)
    });

    if order.len() > 2 {
        let first = bounds[order[0]];
        let origin = (first.max_x as f64, first.max_y as f64);
        order[1..].sort_by(|left, right| {
            let left = squared_distance(origin, bounds[*left]);
            let right = squared_distance(origin, bounds[*right]);
            if left < right {
                Ordering::Less
            } else if right < left {
                Ordering::Greater
            } else {
                Ordering::Equal
            }
        });
    }

    let mut candidates = candidates.into_iter().map(Some).collect::<Vec<_>>();
    order
        .into_iter()
        .map(|index| {
            candidates[index]
                .take()
                .expect("candidate permutation contains every index once")
        })
        .collect()
}

fn squared_distance(origin: (f64, f64), bounds: SourceBounds) -> f64 {
    let x = origin.0 - bounds.min_x as f64;
    let y = origin.1 - bounds.min_y as f64;
    x * x + y * y
}

fn candidate_bounds(candidate: &CandidateSurface) -> SourceBounds {
    let Some(first) = candidate.new_polygons.first() else {
        return SourceBounds::default();
    };
    let mut bounds = polygon_bounds(first);
    for polygon in &candidate.new_polygons[1..] {
        let next = polygon_bounds(polygon);
        if next.defined {
            if bounds.defined {
                bounds.min_x = bounds.min_x.min(next.min_x);
                bounds.min_y = bounds.min_y.min(next.min_y);
                bounds.max_x = bounds.max_x.max(next.max_x);
                bounds.max_y = bounds.max_y.max(next.max_y);
            } else {
                bounds = next;
            }
        }
    }
    bounds
}

fn polygon_bounds(polygon: &Polygon) -> SourceBounds {
    let first = polygon.points()[0];
    let mut bounds = SourceBounds {
        min_x: first.x(),
        min_y: first.y(),
        max_x: first.x(),
        max_y: first.y(),
        defined: false,
    };
    for point in &polygon.points()[1..] {
        bounds.min_x = bounds.min_x.min(point.x());
        bounds.min_y = bounds.min_y.min(point.y());
        bounds.max_x = bounds.max_x.max(point.x());
        bounds.max_y = bounds.max_y.max(point.y());
    }
    bounds.defined = bounds.min_x < bounds.max_x && bounds.min_y < bounds.max_y;
    bounds
}

#[cfg(test)]
mod tests;
