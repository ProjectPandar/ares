//! TriangleSelector.cpp::perform_split / triangle_midpoint_or_allocate.

use super::{Selector, Triangle};

impl Selector {
    fn midpoint_or_allocate(&mut self, neighbor: Option<usize>, a: u32, b: u32) -> u32 {
        if let Some(midpoint) = self.midpoint(neighbor, a, b) {
            return midpoint;
        }
        let a = self.vertices[a as usize];
        let b = self.vertices[b as usize];
        let midpoint = std::array::from_fn(|axis| 0.5 * (a[axis] + b[axis]));
        let index = self.vertices.len() as u32;
        self.vertices.push(midpoint);
        index
    }

    pub(super) fn perform_split(&mut self, index: usize, neighbors: [Option<usize>; 3]) {
        let triangle = self.triangles[index];
        let i = triangle.special();
        let j = (i + 1) % 3;
        let k = (i + 2) % 3;
        let [a, b, c] = [i, j, k].map(|corner| triangle.vertices[corner]);
        let (children, count) = match triangle.splits() {
            1 => {
                let m = self.midpoint_or_allocate(neighbors[j], c, b);
                ([[a, b, m], [m, c, a], [0; 3], [0; 3]], 2)
            }
            2 => {
                let ab = self.midpoint_or_allocate(neighbors[i], b, a);
                let ca = self.midpoint_or_allocate(neighbors[k], a, c);
                ([[a, ab, ca], [ab, b, ca], [b, c, ca], [0; 3]], 3)
            }
            3 => {
                let ab = self.midpoint_or_allocate(neighbors[0], b, a);
                let bc = self.midpoint_or_allocate(neighbors[1], c, b);
                let ca = self.midpoint_or_allocate(neighbors[2], a, c);
                ([[a, ab, ca], [ab, b, bc], [bc, c, ca], [ab, bc, ca]], 4)
            }
            _ => unreachable!("validated split triangle"),
        };
        for (child, vertices) in children.into_iter().take(count).enumerate() {
            let child_index = self.triangles.len();
            self.triangles.push(Triangle::new(vertices));
            self.triangles[index].children[child] = child_index;
        }
    }
}
