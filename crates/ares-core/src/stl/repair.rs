mod connectivity;
mod normals;
mod volume;

use crate::{Point3, Triangle};

pub(super) struct Facet {
    pub(super) normal: [f32; 3],
    pub(super) vertices: [Point3; 3],
}

#[derive(Clone, Copy, Default)]
struct Neighbors {
    facet: [usize; 3],
    vnot: [usize; 3],
}

pub(super) fn into_triangles(mut facets: Vec<Facet>) -> Vec<Triangle> {
    // Classify the complete input before any direction or normal mutation.
    if let Some(neighbors) = connectivity::exact_neighbors(&facets) {
        fix_exact(&mut facets, neighbors);
    }
    triangles(facets)
}

pub(super) fn into_closed_triangles(mut facets: Vec<Facet>) -> Option<Vec<Triangle>> {
    let neighbors = connectivity::exact_neighbors(&facets)?;
    fix_exact(&mut facets, neighbors);
    Some(triangles(facets))
}

fn fix_exact(facets: &mut [Facet], mut neighbors: Vec<Neighbors>) {
    normals::fix_directions(facets, &mut neighbors);
    for facet in facets.iter_mut() {
        normals::check_normal(facet, true);
    }
    if volume::calculate(facets) < 0.0 {
        for index in 0..facets.len() {
            normals::reverse_facet(facets, &mut neighbors, index);
            facets[index].normal = normals::calculate(&facets[index]);
        }
    }
}

fn triangles(facets: Vec<Facet>) -> Vec<Triangle> {
    facets
        .into_iter()
        .map(|facet| Triangle::new(facet.vertices))
        .collect()
}
