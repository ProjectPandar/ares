//! Source per-layer float-line collision cache with forced zero query radius.

use super::{Smoothing, TreeModelVolumes};
use crate::geometry::{Coord, FloatLineDistanceTree};

impl Smoothing<'_> {
    pub(super) fn collision_cache(
        &self,
        volumes: &TreeModelVolumes<'_>,
    ) -> Vec<Option<FloatLineDistanceTree>> {
        let mut minimum = Vec::with_capacity(1024);
        for link in &self.links.elements {
            minimum.resize(minimum.len().max(link.layer + 1), Coord::MAX);
            minimum[link.layer] = minimum[link.layer].min(
                self.graph[link.layer][link.index]
                    .state
                    .actual_radius(self.config),
            );
        }
        minimum
            .into_iter()
            .enumerate()
            .map(|(layer, radius)| {
                if radius == Coord::MAX {
                    return None;
                }
                let (_, polygons) = volumes
                    .collision_lower_bound_area(layer, 0)
                    .expect("organic collision cache contains a zero-radius layer");
                let lines = polygons
                    .iter()
                    .flat_map(|polygon| polygon.lines())
                    .map(|line| {
                        [
                            [
                                self.scale.unscale(line.a.x()),
                                self.scale.unscale(line.a.y()),
                            ],
                            [
                                self.scale.unscale(line.b.x()),
                                self.scale.unscale(line.b.y()),
                            ],
                        ]
                    })
                    .collect();
                Some(FloatLineDistanceTree::new(lines))
            })
            .collect()
    }
}
