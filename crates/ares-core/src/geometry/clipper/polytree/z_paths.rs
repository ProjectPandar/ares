use super::PolyTree;
use crate::geometry::clipper::{
    types::PolyNodeContour,
    z::{KernelPoint, ZPath},
};

impl PolyTree {
    pub(in crate::geometry) fn into_z_paths(self) -> Vec<ZPath> {
        let mut paths = Vec::new();
        let mut pending = self.children.iter().rev().copied().collect::<Vec<_>>();
        while let Some(node_id) = pending.pop() {
            let node = &self.nodes[node_id.0];
            for child in node.children.iter().rev() {
                pending.push(*child);
            }
            let Some(contour) = node.contour.as_ref() else {
                continue;
            };
            let points = match contour {
                PolyNodeContour::Closed(path) => path.points(),
                PolyNodeContour::Open(path) => path.points(),
            };
            if points.is_empty() {
                continue;
            }
            let z = node
                .z
                .as_ref()
                .expect("Z execution materializes a parallel Z contour");
            debug_assert_eq!(points.len(), z.len());
            paths.push(
                points
                    .iter()
                    .zip(z)
                    .map(|(&xy, &z)| KernelPoint { xy, z })
                    .collect(),
            );
        }
        paths
    }
}
