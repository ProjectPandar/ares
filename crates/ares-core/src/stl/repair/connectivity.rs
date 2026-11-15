use std::collections::{HashMap, hash_map::Entry};

use super::{Facet, Neighbors};
use crate::Point3;

struct Edge {
    facet: usize,
    slot: usize,
    backwards: bool,
    paired: bool,
}

// admesh/connect.cpp: HashEdge::load_exact and record_neighbors, two-use domain.
pub(super) fn exact_neighbors(facets: &[Facet]) -> Option<Vec<Neighbors>> {
    let mut edges = HashMap::with_capacity(facets.len() * 3);
    let mut neighbors = vec![Neighbors::default(); facets.len()];
    for (index, facet) in facets.iter().enumerate() {
        if degenerate(facet) {
            return None;
        }
        for slot in 0..3 {
            let a = coordinates(facet.vertices[slot]);
            let b = coordinates(facet.vertices[(slot + 1) % 3]);
            let lower = if a[0] != b[0] {
                a[0] < b[0]
            } else if a[1] != b[1] {
                a[1] < b[1]
            } else {
                a[2] < b[2]
            };
            let (first, second) = if lower { (a, b) } else { (b, a) };
            let key = [first, second].map(coordinate_key);
            let edge = Edge {
                facet: index,
                slot,
                backwards: !lower,
                paired: false,
            };
            match edges.entry(key) {
                Entry::Vacant(entry) => {
                    entry.insert(edge);
                }
                Entry::Occupied(mut entry) => {
                    record_neighbors(entry.get_mut(), edge, &mut neighbors)?;
                }
            }
        }
    }
    edges.values().all(|edge| edge.paired).then_some(neighbors)
}

fn record_neighbors(previous: &mut Edge, edge: Edge, neighbors: &mut [Neighbors]) -> Option<()> {
    if previous.paired || previous.facet == edge.facet {
        return None;
    }
    let direction = if previous.backwards == edge.backwards {
        3
    } else {
        0
    };
    neighbors[edge.facet].facet[edge.slot] = previous.facet;
    neighbors[edge.facet].vnot[edge.slot] = (previous.slot + 2) % 3 + direction;
    neighbors[previous.facet].facet[previous.slot] = edge.facet;
    neighbors[previous.facet].vnot[previous.slot] = (edge.slot + 2) % 3 + direction;
    previous.paired = true;
    Some(())
}

fn coordinate_key(point: [f32; 3]) -> [u32; 3] {
    point.map(|coordinate| {
        if coordinate == 0.0 {
            0
        } else {
            coordinate.to_bits()
        }
    })
}

fn coordinates(point: Point3) -> [f32; 3] {
    [point.x, point.y, point.z]
}

fn degenerate(facet: &Facet) -> bool {
    let [a, b, c] = facet
        .vertices
        .map(|point| coordinates(point).map(f64::from));
    let u: [f64; 3] = std::array::from_fn(|axis| b[axis] - a[axis]);
    let v: [f64; 3] = std::array::from_fn(|axis| c[axis] - a[axis]);
    [
        u[1] * v[2] - u[2] * v[1],
        u[2] * v[0] - u[0] * v[2],
        u[0] * v[1] - u[1] * v[0],
    ] == [0.0; 3]
}
