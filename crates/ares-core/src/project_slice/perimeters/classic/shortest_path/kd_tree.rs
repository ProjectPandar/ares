const EPSILON: f64 = 1e-4;
const NONE: usize = usize::MAX;

pub(super) struct KdTree {
    nodes: Vec<usize>,
}

impl KdTree {
    pub(super) fn new(points: &[[f64; 2]]) -> Self {
        let mut indices: Vec<_> = (0..points.len()).collect();
        let mut nodes = vec![NONE; (indices.len() + 1).next_power_of_two()];
        if !indices.is_empty() {
            BuildContext {
                nodes: &mut nodes,
                input: &mut indices,
                points,
            }
            .build(0, 0, (0, points.len() - 1));
        }
        Self { nodes }
    }

    pub(super) fn find_closest(
        &self,
        points: &[[f64; 2]],
        point: [f64; 2],
        mut filter: impl FnMut(usize) -> bool,
    ) -> usize {
        let mut search = Search {
            tree: self,
            points,
            point,
            filter: &mut filter,
            result: (NONE, f64::MAX),
        };
        search.visit(0, 0);
        search.result.0
    }
}

struct Search<'a, F> {
    tree: &'a KdTree,
    points: &'a [[f64; 2]],
    point: [f64; 2],
    filter: &'a mut F,
    result: (usize, f64),
}

impl<F: FnMut(usize) -> bool> Search<'_, F> {
    fn visit(&mut self, node: usize, dimension: usize) {
        if node >= self.tree.nodes.len() || self.tree.nodes[node] == NONE {
            return;
        }
        let idx = self.tree.nodes[node];
        if (self.filter)(idx) {
            let dx = self.point[0] - self.points[idx][0];
            let dy = self.point[1] - self.points[idx][1];
            let distance = dx * dx + dy * dy;
            if distance <= self.result.1 {
                self.result = (idx, distance);
            }
        }
        let delta = self.point[dimension] - self.points[idx][dimension];
        let both = delta * delta < self.result.1 + EPSILON;
        let next_dimension = (dimension + 1) & 1;
        if both || delta <= 0.0 {
            self.visit(node * 2 + 1, next_dimension);
        }
        if both || delta > 0.0 {
            self.visit(node * 2 + 2, next_dimension);
        }
    }
}

struct BuildContext<'a> {
    nodes: &'a mut [usize],
    input: &'a mut [usize],
    points: &'a [[f64; 2]],
}

impl BuildContext<'_> {
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
        let next_dimension = (dimension + 1) & 1;
        if center > left {
            self.build(node * 2 + 1, next_dimension, (left, center - 1));
        }
        self.build(node * 2 + 2, next_dimension, (center + 1, right));
    }

    fn advance_left(&self, mut index: usize, dimension: usize, pivot: f64) -> usize {
        while self.points[self.input[index]][dimension] < pivot {
            index += 1;
        }
        index
    }

    fn retreat_right(&self, mut index: usize, left: usize, dimension: usize, pivot: f64) -> usize {
        while self.points[self.input[index]][dimension] > pivot && left < index {
            index -= 1;
        }
        index
    }

    fn partition(&mut self, dimension: usize, range: (usize, usize), k: usize) {
        let (mut left, mut right) = range;
        while left < right {
            let center = (left + right) / 2;
            let mut left_value = self.points[self.input[left]][dimension];
            let mut center_value = self.points[self.input[center]][dimension];
            let mut right_value = self.points[self.input[right]][dimension];
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
            match k.cmp(&pivot_index) {
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
        pivot: f64,
    ) -> usize {
        let (left, right) = range;
        let mut i = left;
        let mut j = right - 1;
        self.input.swap(center, j);
        loop {
            i = self.advance_left(i + 1, dimension, pivot);
            j = self.retreat_right(j - 1, i, dimension, pivot);
            if i >= j {
                break;
            }
            self.input.swap(i, j);
        }
        self.input.swap(i, right - 1);
        i
    }
}
