//! Faithful port of `KDTreeIndirect` (`libslic3r/KDTreeIndirect.hpp`) for the
//! seam-candidate nearest query: a balanced 3-D tree whose build order
//! (median-of-three quickselect partition, heap-indexed nodes) and visit order
//! (root → left/right by descent mask) must match upstream so equal-distance
//! candidates resolve to the same index (`find_closest_point`, k=1:
//! `KDTreeIndirect.hpp:266-271` returns the last visited equal-distance
//! result). Upstream builds `KDTreeIndirect<3, float,
//! SeamCandidateCoordinateFunctor>` (`SeamPlacer.hpp:92`).

const NPOS: usize = usize::MAX;
const DIMS: usize = 3;

pub(in crate::project_slice) struct KdTree {
    /// Heap-indexed node → candidate index (`NPOS` for empty slots).
    nodes: Vec<usize>,
}

impl KdTree {
    /// `KDTreeIndirect::build` (`KDTreeIndirect.hpp:40-56`): allocate a full
    /// binary tree and partition `0..n` recursively.
    pub(in crate::project_slice) fn build(
        n: usize,
        coordinate: impl Fn(usize, usize) -> f32,
    ) -> Self {
        if n == 0 {
            return Self { nodes: Vec::new() };
        }
        let mut input: Vec<usize> = (0..n).collect();
        let capacity = (n + 1).next_power_of_two() * 2;
        let mut tree = Self {
            nodes: vec![NPOS; capacity],
        };
        tree.build_recursive(&mut input, (0, 0), (0, n - 1), &coordinate);
        tree
    }

    /// `build_recursive` (`KDTreeIndirect.hpp:81-108`): split
    /// `input[left..=right]` at the axis-aligned median, write the median node
    /// index to `nodes[node]`, and recurse left then right.
    fn build_recursive(
        &mut self,
        input: &mut [usize],
        (node, dimension): (usize, usize),
        (left, right): (usize, usize),
        coordinate: &dyn Fn(usize, usize) -> f32,
    ) {
        if left > right || node >= self.nodes.len() {
            return;
        }
        if left == right {
            self.nodes[node] = input[left];
            return;
        }
        let center = (left + right) / 2;
        partition_input(input, dimension, (left, right), center, coordinate);
        self.nodes[node] = input[center];
        let next_dimension = (dimension + 1) % DIMS;
        if center > left {
            self.build_recursive(
                input,
                (node * 2 + 1, next_dimension),
                (left, center - 1),
                coordinate,
            );
        }
        // Upstream recurses the right subtree from `center + 1`
        // (`KDTreeIndirect.hpp:107`); `center` would re-insert the
        // median and could loop on duplicates.
        self.build_recursive(
            input,
            (node * 2 + 2, next_dimension),
            (center + 1, right),
            coordinate,
        );
    }

    /// `find_closest_point` k=1 (`KDTreeIndirect.hpp:200-251,266-271`):
    /// visit nodes in tree order, inserting each visited candidate by
    /// `lower_bound` on distance — an equal-distance result ROTATES the
    /// previous equal result out, so the LAST visited equal candidate wins.
    pub(in crate::project_slice) fn find_closest(
        &self,
        point: [f32; 3],
        coordinate: &dyn Fn(usize, usize) -> f32,
    ) -> Option<usize> {
        let mut best: (usize, f32) = (NPOS, f32::MAX);
        self.visit_recursive((0, 0), point, coordinate, &mut best);
        (best.0 != NPOS).then_some(best.0)
    }

    /// `visit_recursive` + `Visitor::operator()` + `descent_mask`
    /// (`KDTreeIndirect.hpp:169-190,221-244,59-69`). `EPSILON` = `1e-4`
    /// (`libslic3r.h:52`) widens the descent sphere radius.
    fn visit_recursive(
        &self,
        (node, dimension): (usize, usize),
        point: [f32; 3],
        coordinate: &dyn Fn(usize, usize) -> f32,
        best: &mut (usize, f32),
    ) {
        if node >= self.nodes.len() || self.nodes[node] == NPOS {
            return;
        }
        let idx = self.nodes[node];
        // Visitor: distance + lower_bound insert. For K=1 the insert
        // replaces the slot iff `dist <= best.1` (lower_bound finds the
        // first slot whose value >= dist; `it != end` ⇒ dist <= current).
        let mut dist = 0.0f32;
        for (dim, &value) in point.iter().enumerate().take(DIMS) {
            let d = value - coordinate(idx, dim);
            dist += d * d;
        }
        if dist <= best.1 {
            *best = (idx, dist);
        }
        // descent_mask(point_coord, search_radius = best distance, dim).
        let diff = point[dimension] - coordinate(idx, dimension);
        let (go_left, go_right) = if diff * diff < best.1 + 1e-4 {
            (true, true)
        } else if diff > 0.0 {
            (false, true)
        } else {
            (true, false)
        };
        let next_dimension = (dimension + 1) % DIMS;
        if go_left {
            self.visit_recursive((node * 2 + 1, next_dimension), point, coordinate, best);
        }
        if go_right {
            self.visit_recursive((node * 2 + 2, next_dimension), point, coordinate, best);
        }
    }
}

/// `partition_input` (`KDTreeIndirect.hpp:111-160`): median-of-three
/// quickselect around `k` on `dimension`.
fn partition_input(
    input: &mut [usize],
    dimension: usize,
    (mut left, mut right): (usize, usize),
    k: usize,
    coordinate: &dyn Fn(usize, usize) -> f32,
) {
    while left < right {
        let center = (left + right) / 2;
        let mut left_value = coordinate(input[left], dimension);
        let mut center_value = coordinate(input[center], dimension);
        let mut right_value = coordinate(input[right], dimension);
        if left_value > center_value {
            input.swap(left, center);
            std::mem::swap(&mut left_value, &mut center_value);
        }
        if left_value > right_value {
            input.swap(left, right);
            right_value = left_value;
        }
        if center_value > right_value {
            input.swap(center, right);
            center_value = right_value;
        }
        let pivot = center_value;
        if right <= left + 2 {
            break;
        }
        let mut i = left;
        let mut j = right - 1;
        input.swap(center, j);
        loop {
            i += 1;
            while coordinate(input[i], dimension) < pivot {
                i += 1;
            }
            // Upstream `while (coordinate(input[--j]) > pivot && i < j)`:
            // the decrement happens before the comparison; `i < j`
            // short-circuits before `j` can underflow past `i >= left+1`.
            j -= 1;
            while coordinate(input[j], dimension) > pivot && i < j {
                j -= 1;
            }
            if i >= j {
                break;
            }
            input.swap(i, j);
        }
        input.swap(i, right - 1);
        if k < i {
            right = i - 1;
        } else if k == i {
            break;
        } else {
            left = i + 1;
        }
    }
}
