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

pub(super) fn into_triangles(mut facets: Vec<Facet>) -> (Vec<Triangle>, Option<f32>) {
    let mut solid_volume = None;
    // Classify the complete input before any direction or normal mutation.
    if let Some(mut neighbors) = connectivity::exact_neighbors(&facets) {
        normals::fix_directions(&mut facets, &mut neighbors);
        for facet in &mut facets {
            normals::check_normal(facet, true);
        }
        let volume = volume::calculate(&facets);
        solid_volume = Some(volume.abs());
        if volume < 0.0 {
            for index in 0..facets.len() {
                normals::reverse_facet(&mut facets, &mut neighbors, index);
                facets[index].normal = normals::calculate(&facets[index]);
            }
        }
    }
    let triangles = facets
        .into_iter()
        .map(|facet| Triangle::new(facet.vertices))
        .collect();
    (triangles, solid_volume)
}
