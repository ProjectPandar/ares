mod bvh;

pub(super) use bvh::TriangleBvh;

use super::mesh::Vec3;
const NONE: usize = usize::MAX;
const KD_EPSILON: f32 = 1e-4;

pub(super) struct PointKdTree {
    nodes: Vec<usize>,
}

impl PointKdTree {
    pub(super) fn new(points: &[Vec3]) -> Self {
        let mut indices = (0..points.len()).collect::<Vec<_>>();
        let mut nodes = vec![NONE; (indices.len() + 1).next_power_of_two()];
        if !indices.is_empty() {
            PointBuild {
                nodes: &mut nodes,
                input: &mut indices,
                points,
            }
            .build(0, 0, (0, points.len() - 1));
        }
        Self { nodes }
    }

    pub(super) fn in_radius(&self, points: &[Vec3], target: Vec3, radius: f32) -> Vec<usize> {
        let mut search = PointRadiusSearch {
            points,
            target,
            radius_squared: radius * radius,
            output: Vec::new(),
        };
        search.visit(self, 0, 0);
        search.output
    }
}

struct PointRadiusSearch<'a> {
    points: &'a [Vec3],
    target: Vec3,
    radius_squared: f32,
    output: Vec<usize>,
}

impl PointRadiusSearch<'_> {
    fn visit(&mut self, tree: &PointKdTree, node: usize, dimension: usize) {
        if node >= tree.nodes.len() || tree.nodes[node] == NONE {
            return;
        }
        let index = tree.nodes[node];
        if (self.target - self.points[index]).norm_squared() < self.radius_squared {
            self.output.push(index);
        }
        let delta = self.target.axis(dimension) - self.points[index].axis(dimension);
        let both = delta * delta < self.radius_squared + KD_EPSILON;
        let next_dimension = (dimension + 1) % 3;
        if both || delta <= 0.0 {
            self.visit(tree, node * 2 + 1, next_dimension);
        }
        if both || delta > 0.0 {
            self.visit(tree, node * 2 + 2, next_dimension);
        }
    }
}

struct PointBuild<'a> {
    nodes: &'a mut [usize],
    input: &'a mut [usize],
    points: &'a [Vec3],
}

impl PointBuild<'_> {
    fn build(&mut self, node: usize, dimension: usize, range: (usize, usize)) {
        let (left, right) = range;
        if left > right {
            return;
        }
        if left == right {
            self.nodes[node] = self.input[left];
            return;
        }
        let center = (left + right) / 2;
        self.partition(dimension, range, center);
        self.nodes[node] = self.input[center];
        let next_dimension = (dimension + 1) % 3;
        if center > left {
            self.build(node * 2 + 1, next_dimension, (left, center - 1));
        }
        self.build(node * 2 + 2, next_dimension, (center + 1, right));
    }

    fn partition(&mut self, dimension: usize, range: (usize, usize), target: usize) {
        let (mut left, mut right) = range;
        while left < right {
            let center = (left + right) / 2;
            let mut left_value = self.points[self.input[left]].axis(dimension);
            let mut center_value = self.points[self.input[center]].axis(dimension);
            let mut right_value = self.points[self.input[right]].axis(dimension);
            if left_value > center_value {
                self.input.swap(left, center);
                std::mem::swap(&mut left_value, &mut center_value);
            }
            if left_value > right_value {
                self.input.swap(left, right);
                right_value = left_value;
            }
            if center_value > right_value {
                self.input.swap(center, right);
                center_value = right_value;
            }
            let pivot = center_value;
            if right <= left + 2 {
                break;
            }
            let pivot_index = self.partition_pass(center, (left, right), dimension, pivot);
            match target.cmp(&pivot_index) {
                std::cmp::Ordering::Less => right = pivot_index - 1,
                std::cmp::Ordering::Equal => break,
                std::cmp::Ordering::Greater => left = pivot_index + 1,
            }
        }
    }

    fn partition_pass(
        &mut self,
        center: usize,
        range: (usize, usize),
        dimension: usize,
        pivot: f32,
    ) -> usize {
        let (left, right) = range;
        let mut first = left;
        let mut last = right - 1;
        self.input.swap(center, last);
        loop {
            first += 1;
            while self.points[self.input[first]].axis(dimension) < pivot {
                first += 1;
            }
            last -= 1;
            while self.points[self.input[last]].axis(dimension) > pivot && first < last {
                last -= 1;
            }
            if first >= last {
                break;
            }
            self.input.swap(first, last);
        }
        self.input.swap(first, right - 1);
        first
    }
}
