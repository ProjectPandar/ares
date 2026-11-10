//! `MinimumSpanningTree.cpp::prim/adjacent_nodes`, Linux reference ordering.

use crate::geometry::{CoordinateScale, Point};

pub(super) struct MinimumSpanningTree {
    adjacency: Vec<Vec<usize>>,
}

impl MinimumSpanningTree {
    pub(super) fn new(vertices: &[Point], scale: CoordinateScale) -> Self {
        let mut adjacency = vec![Vec::new(); vertices.len()];
        if vertices.len() < 2 {
            return Self { adjacency };
        }
        let mut distances = vertices
            .iter()
            .map(|&point| Some(distance_squared(vertices[0], point, scale)))
            .collect::<Vec<_>>();
        distances[0] = None;
        let mut closest_to = vec![0; vertices.len()];
        for _ in 1..vertices.len() {
            // MinimumSpanningTree.cpp:66-77: equal distances fall back to Point operator<.
            // Coincident points keep the map order: reserve(N), identity pointer hash and
            // 16-byte Point stride yield distinct odd-prime buckets, so it is reverse insertion.
            let closest = distances
                .iter()
                .enumerate()
                .skip(1)
                .rev()
                .filter_map(|(index, distance)| distance.map(|distance| (index, distance)))
                .min_by(|left, right| {
                    let coordinates = |index: usize| (vertices[index].x(), vertices[index].y());
                    left.1
                        .total_cmp(&right.1)
                        .then_with(|| coordinates(left.0).cmp(&coordinates(right.0)))
                })
                .expect("a spanning tree has an unconnected vertex")
                .0;
            let other = closest_to[closest];
            adjacency[closest].push(other);
            adjacency[other].push(closest);
            distances[closest] = None;
            update_distances(vertices, scale, closest, &mut distances, &mut closest_to);
        }
        Self { adjacency }
    }

    pub(super) fn adjacent_nodes(&self, vertex: usize) -> &[usize] {
        &self.adjacency[vertex]
    }
}

fn update_distances(
    vertices: &[Point],
    scale: CoordinateScale,
    closest: usize,
    distances: &mut [Option<f64>],
    closest_to: &mut [usize],
) {
    for (index, slot) in distances.iter_mut().enumerate().skip(1).rev() {
        let Some(old) = *slot else { continue };
        let distance = distance_squared(vertices[closest], vertices[index], scale);
        if distance < old {
            *slot = Some(distance);
            closest_to[index] = closest;
        }
    }
}

fn distance_squared(first: Point, second: Point, scale: CoordinateScale) -> f64 {
    let x = scale.unscale(first.x() - second.x());
    let y = scale.unscale(first.y() - second.y());
    x * x + y * y
}
