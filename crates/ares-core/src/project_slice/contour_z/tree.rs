//! AABBTreeIndirect.hpp:38-217,672-717: serial Tree3f and source face order.
mod partition;
#[derive(Clone, Copy, Debug, PartialEq)]
pub(super) struct Node {
    pub(super) index: usize,
    pub(super) bounds: [[f32; 3]; 2],
}
pub(super) const UNUSED: usize = usize::MAX;
pub(super) const INNER: usize = usize::MAX - 1;
#[derive(Debug, PartialEq)]
pub(super) struct Tree {
    pub(super) nodes: Vec<Node>,
}
#[derive(Clone, Copy)]
struct Input {
    index: usize,
    bounds: [[f32; 3]; 2],
    centroid: [f32; 3],
}
impl Tree {
    pub(super) fn build(vertices: &[[f32; 3]], faces: &[[usize; 3]]) -> Self {
        let mut input = faces
            .iter()
            .enumerate()
            .map(|(index, face)| {
                let [a, b, c] = face.map(|i| vertices[i]);
                Input {
                    index,
                    bounds: [
                        std::array::from_fn(|i| a[i].min(b[i]).min(c[i])),
                        std::array::from_fn(|i| a[i].max(b[i]).max(c[i])),
                    ],
                    centroid: std::array::from_fn(|i| (a[i] + b[i] + c[i]) * (1.0f32 / 3.0)),
                }
            })
            .collect::<Vec<_>>();
        let mut tree = Self { nodes: Vec::new() };
        if !input.is_empty() {
            tree.nodes.resize(
                input.len().next_power_of_two() * 2 - 1,
                Node {
                    index: UNUSED,
                    bounds: [[0.; 3]; 2],
                },
            );
            let last = input.len() - 1;
            tree.build_recursive(&mut input, 0, 0, last);
        }
        tree
    }
    fn build_recursive(&mut self, input: &mut [Input], node: usize, left: usize, right: usize) {
        if left == right {
            self.nodes[node] = Node {
                index: input[left].index,
                bounds: input[left].bounds,
            };
            return;
        }
        let mut bounds = input[left].bounds;
        for entry in &input[left + 1..=right] {
            let [minimum, maximum] = &mut bounds;
            for (axis, (minimum, maximum)) in minimum.iter_mut().zip(maximum).enumerate() {
                *minimum = minimum.min(entry.bounds[0][axis]);
                *maximum = maximum.max(entry.bounds[1][axis]);
            }
        }
        let mut dimension = 0;
        for axis in 1..3 {
            if bounds[1][axis] - bounds[0][axis] > bounds[1][dimension] - bounds[0][dimension] {
                dimension = axis;
            }
        }
        let center = (left + right) / 2;
        partition::partition(input, dimension, left, right, center);
        self.nodes[node] = Node {
            index: INNER,
            bounds,
        };
        self.build_recursive(input, node * 2 + 1, left, center);
        self.build_recursive(input, node * 2 + 2, center + 1, right);
    }
}
