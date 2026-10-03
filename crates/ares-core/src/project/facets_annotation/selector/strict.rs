//! TriangleSelector.cpp::get_facets_split_by_tjoints.

use super::Selector;

impl Selector {
    pub(super) fn split_tjoints(
        &self,
        vertices: [u32; 3],
        neighbors: [Option<usize>; 3],
        output: &mut Vec<[u32; 3]>,
    ) {
        let mut pending = vec![(vertices, neighbors)];
        while let Some((vertices, neighbors)) = pending.pop() {
            let midpoints: [Option<u32>; 3] = std::array::from_fn(|edge| {
                self.midpoint(neighbors[edge], vertices[(edge + 1) % 3], vertices[edge])
            });
            let splits = midpoints.iter().flatten().count();
            let partition = |edge, second| {
                self.neighbor_child(
                    neighbors[edge],
                    vertices[(edge + 1) % 3],
                    vertices[edge],
                    second,
                )
            };
            match splits {
                0 => output.push(vertices),
                1 => {
                    let i = (midpoints.iter().position(Option::is_some).unwrap() + 2) % 3;
                    let j = (i + 1) % 3;
                    let k = (i + 2) % 3;
                    let middle = midpoints[j].unwrap();
                    pending.push((
                        [middle, vertices[k], vertices[i]],
                        [partition(j, false), neighbors[k], None],
                    ));
                    pending.push((
                        [vertices[i], vertices[j], middle],
                        [neighbors[i], partition(j, true), None],
                    ));
                }
                2 => {
                    let i = (midpoints.iter().position(Option::is_none).unwrap() + 2) % 3;
                    let j = (i + 1) % 3;
                    let k = (i + 2) % 3;
                    let mi = midpoints[i].unwrap();
                    let mk = midpoints[k].unwrap();
                    pending.push((
                        [vertices[j], vertices[k], mk],
                        [neighbors[j], partition(k, true), None],
                    ));
                    pending.push(([mi, vertices[j], mk], [partition(i, false), None, None]));
                    pending.push((
                        [vertices[i], mi, mk],
                        [partition(i, true), None, partition(k, false)],
                    ));
                }
                3 => {
                    let [a, b, c] = vertices;
                    let [ab, bc, ca] = midpoints.map(Option::unwrap);
                    pending.push(([ab, bc, ca], [None; 3]));
                    pending.push(([bc, c, ca], [partition(1, false), partition(2, true), None]));
                    pending.push(([ab, b, bc], [partition(0, false), partition(1, true), None]));
                    pending.push(([a, ab, ca], [partition(0, true), None, partition(2, false)]));
                }
                _ => unreachable!("three triangle edges"),
            }
        }
    }
}
