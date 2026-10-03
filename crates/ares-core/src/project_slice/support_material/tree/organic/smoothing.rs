//! Active `TreeSupport3D.cpp::organic_smooth_branches_avoid_collisions`.

use std::collections::VecDeque;

use super::super::model_volumes::TreeModelVolumes;
use super::{TreeSettings, support_element::SupportElement};
use crate::{
    geometry::{CoordinateScale, Point},
    project_slice::parameters::SlicingParameters,
};

mod cache;
mod constraints;
mod links;
mod movement;
mod spheres;

pub(super) use links::BranchLinks;

struct Smoothing<'a> {
    graph: &'a [VecDeque<SupportElement>],
    links: &'a BranchLinks,
    config: &'a TreeSettings,
    slicing: &'a SlicingParameters,
    scale: CoordinateScale,
}

#[derive(Clone, Copy)]
struct Sphere {
    link: usize,
    locked: bool,
    radius: f32,
    position: [f32; 3],
    previous: [f32; 3],
    min_z: f32,
    max_z: f32,
    layer_begin: u32,
    layer_end: u32,
}

pub(super) fn organic_smooth_branches_avoid_collisions(
    volumes: &TreeModelVolumes<'_>,
    config: &TreeSettings,
    slicing: &SlicingParameters,
    graph: &mut [VecDeque<SupportElement>],
    scale: CoordinateScale,
) -> BranchLinks {
    let links = BranchLinks::new(graph);
    let spheres = {
        let context = Smoothing {
            graph,
            links: &links,
            config,
            slicing,
            scale,
        };
        let cache = context.collision_cache(volumes);
        let mut spheres = context.spheres(cache.len());
        for _ in 0..100 {
            if context.iteration(&mut spheres, &cache) == 0 {
                break;
            }
        }
        spheres
    };
    for (link, sphere) in links.elements.iter().zip(spheres) {
        graph[link.layer][link.index].state.result_on_layer = Some(Point::new(
            (sphere.position[0] / scale.factor() as f32) as i64,
            (sphere.position[1] / scale.factor() as f32) as i64,
        ));
    }
    links
}

impl Smoothing<'_> {
    fn iteration(
        &self,
        spheres: &mut [Sphere],
        cache: &[Option<crate::geometry::FloatLineDistanceTree>],
    ) -> usize {
        for sphere in spheres.iter_mut() {
            sphere.previous = sphere.position;
        }
        let mut moved = 0;
        for index in 0..spheres.len() {
            let mut sphere = spheres[index];
            if sphere.locked {
                continue;
            }
            moved += usize::from(self.move_sphere(&mut sphere, spheres, cache));
            spheres[index] = sphere;
        }
        moved
    }

    fn element(&self, sphere: &Sphere) -> &SupportElement {
        let link = self.links.elements[sphere.link];
        &self.graph[link.layer][link.index]
    }
}
