//! TriangleSelector.cpp::neighbor_child / triangle_midpoint / child_neighbors.

use super::{Selector, Triangle};

impl Selector {
    pub(super) fn neighbor_child(
        &self,
        mut neighbor: Option<usize>,
        a: u32,
        b: u32,
        second: bool,
    ) -> Option<usize> {
        while let Some(index) = neighbor {
            let triangle = self.triangles[index];
            let splits = triangle.splits();
            if splits == 0 {
                return None;
            }
            let edge = triangle
                .vertices
                .iter()
                .position(|&vertex| vertex == a)
                .unwrap();
            debug_assert_eq!(triangle.vertices[(edge + 1) % 3], b);
            let special = triangle.special();
            let child = match splits {
                1 if edge != (special + 1) % 3 => {
                    neighbor = Some(triangle.children[usize::from(edge != special)]);
                    continue;
                }
                1 => usize::from(second),
                2 if edge == (special + 1) % 3 => {
                    neighbor = Some(triangle.children[2]);
                    continue;
                }
                2 if edge == special => usize::from(second),
                2 => [2, 0][usize::from(second)],
                3 => (edge + usize::from(second)) % 3,
                _ => unreachable!("validated split triangle"),
            };
            return Some(triangle.children[child]);
        }
        None
    }

    pub(super) fn midpoint(&self, mut neighbor: Option<usize>, a: u32, b: u32) -> Option<u32> {
        while let Some(index) = neighbor {
            let triangle = self.triangles[index];
            let splits = triangle.splits();
            if splits == 0 {
                return None;
            }
            let edge = triangle
                .vertices
                .iter()
                .position(|&vertex| vertex == a)
                .unwrap();
            debug_assert_eq!(triangle.vertices[(edge + 1) % 3], b);
            let special = triangle.special();
            let (child, corner) = match splits {
                1 if edge != (special + 1) % 3 => {
                    neighbor = Some(triangle.children[usize::from(edge != special)]);
                    continue;
                }
                1 => (0, 2),
                2 if edge == (special + 1) % 3 => {
                    neighbor = Some(triangle.children[2]);
                    continue;
                }
                2 if edge == special => (0, 1),
                2 => (1, 2),
                3 => (edge, if edge == 0 { 1 } else { 2 }),
                _ => unreachable!("validated split triangle"),
            };
            return Some(self.triangles[triangle.children[child]].vertices[corner]);
        }
        None
    }

    pub(super) fn child_neighbors(
        &self,
        triangle: Triangle,
        neighbors: [Option<usize>; 3],
        child: usize,
    ) -> [Option<usize>; 3] {
        let i = triangle.special();
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;
        let vertices = triangle.vertices;
        let sibling = |index| Some(triangle.children[index]);
        let partition = |edge, second| {
            self.neighbor_child(
                neighbors[edge],
                vertices[(edge + 1) % 3],
                vertices[edge],
                second,
            )
        };
        match (triangle.splits(), child) {
            (1, 0) => [neighbors[i], partition(j, true), sibling(1)],
            (1, 1) => [partition(j, false), neighbors[k], sibling(0)],
            (2, 0) => [partition(i, true), sibling(1), partition(k, false)],
            (2, 1) => [partition(i, false), sibling(2), sibling(0)],
            (2, 2) => [neighbors[j], partition(k, true), sibling(1)],
            (3, 0) => [partition(0, true), sibling(3), partition(2, false)],
            (3, 1) => [partition(0, false), partition(1, true), sibling(3)],
            (3, 2) => [partition(1, false), partition(2, true), sibling(3)],
            (3, 3) => [sibling(1), sibling(2), sibling(0)],
            _ => unreachable!("validated child index"),
        }
    }
}
