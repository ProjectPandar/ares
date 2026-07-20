const DIMENSIONS: usize = 2;
const EPSILON: f64 = 1e-4;
const NO_INDEX: usize = usize::MAX;

#[derive(Clone, Copy)]
struct TreePosition {
    node: usize,
    dimension: usize,
}

#[derive(Clone, Copy)]
struct InputRange {
    left: usize,
    right: usize,
}

impl InputRange {
    fn center(self) -> usize {
        self.left + (self.right - self.left) / 2
    }
}

struct ClosestSearch<'a, Filter> {
    points: &'a [[f64; 2]],
    query: [f64; 2],
    filter: Filter,
    result: Option<usize>,
    distance: f64,
}

pub(in crate::geometry) struct KdTree {
    nodes: Vec<usize>,
}

impl KdTree {
    pub(in crate::geometry) fn new(points: &[[f64; 2]]) -> Self {
        if points.is_empty() {
            return Self { nodes: Vec::new() };
        }

        let mut indices = (0..points.len()).collect::<Vec<_>>();
        let mut tree = Self {
            nodes: vec![NO_INDEX; (points.len() + 1).next_power_of_two()],
        };
        tree.build_recursive(
            points,
            &mut indices,
            TreePosition {
                node: 0,
                dimension: 0,
            },
            InputRange {
                left: 0,
                right: points.len() - 1,
            },
        );
        tree
    }

    pub(in crate::geometry) fn closest(
        &self,
        points: &[[f64; 2]],
        query: [f64; 2],
        filter: impl FnMut(usize) -> bool,
    ) -> Option<usize> {
        let mut search = ClosestSearch {
            points,
            query,
            filter,
            result: None,
            distance: f64::MAX,
        };
        self.visit_recursive(
            &mut search,
            TreePosition {
                node: 0,
                dimension: 0,
            },
        );
        search.result
    }

    #[cfg(test)]
    pub(in crate::geometry) fn nodes(&self) -> &[usize] {
        &self.nodes
    }

    fn build_recursive(
        &mut self,
        points: &[[f64; 2]],
        input: &mut [usize],
        position: TreePosition,
        range: InputRange,
    ) {
        if range.left > range.right {
            return;
        }
        if range.left == range.right {
            self.nodes[position.node] = input[range.left];
            return;
        }

        let center = range.center();
        Self::partition_input(points, input, position.dimension, range, center);
        self.nodes[position.node] = input[center];

        let left_child = TreePosition {
            node: position.node * 2 + 1,
            dimension: (position.dimension + 1) % DIMENSIONS,
        };
        if center > range.left {
            self.build_recursive(
                points,
                input,
                left_child,
                InputRange {
                    left: range.left,
                    right: center - 1,
                },
            );
        }
        self.build_recursive(
            points,
            input,
            TreePosition {
                node: left_child.node + 1,
                dimension: left_child.dimension,
            },
            InputRange {
                left: center + 1,
                right: range.right,
            },
        );
    }

    fn partition_input(
        points: &[[f64; 2]],
        input: &mut [usize],
        dimension: usize,
        mut range: InputRange,
        target: usize,
    ) {
        while range.left < range.right {
            let center = range.center();
            let mut left_value = points[input[range.left]][dimension];
            let mut center_value = points[input[center]][dimension];
            let mut right_value = points[input[range.right]][dimension];

            if left_value > center_value {
                input.swap(range.left, center);
                std::mem::swap(&mut left_value, &mut center_value);
            }
            if left_value > right_value {
                input.swap(range.left, range.right);
                right_value = left_value;
            }
            if center_value > right_value {
                input.swap(center, range.right);
                center_value = right_value;
            }

            let pivot = center_value;
            if range.right - range.left <= 2 {
                break;
            }

            let low = Self::partition_around_pivot(points, input, dimension, range, pivot);

            if target < low {
                range.right = low - 1;
            } else if target == low {
                break;
            } else {
                range.left = low + 1;
            }
        }
    }

    fn partition_around_pivot(
        points: &[[f64; 2]],
        input: &mut [usize],
        dimension: usize,
        range: InputRange,
        pivot: f64,
    ) -> usize {
        let mut low = range.left;
        let mut high = range.right - 1;
        input.swap(range.center(), high);
        loop {
            low += 1;
            while points[input[low]][dimension] < pivot {
                low += 1;
            }

            high = Self::scan_high(points, input, dimension, pivot, (low, high));

            if low >= high {
                input.swap(low, range.right - 1);
                return low;
            }
            input.swap(low, high);
        }
    }

    fn scan_high(
        points: &[[f64; 2]],
        input: &[usize],
        dimension: usize,
        pivot: f64,
        (low, mut high): (usize, usize),
    ) -> usize {
        loop {
            high -= 1;
            if !(points[input[high]][dimension] > pivot && low < high) {
                return high;
            }
        }
    }

    fn visit_recursive<Filter: FnMut(usize) -> bool>(
        &self,
        search: &mut ClosestSearch<'_, Filter>,
        position: TreePosition,
    ) {
        if position.node >= self.nodes.len() || self.nodes[position.node] == NO_INDEX {
            return;
        }

        let index = self.nodes[position.node];
        if (search.filter)(index) {
            let dx = search.query[0] - search.points[index][0];
            let dy = search.query[1] - search.points[index][1];
            let distance = dx * dx + dy * dy;
            if distance <= search.distance {
                search.result = Some(index);
                search.distance = distance;
            }
        }

        let plane = search.query[position.dimension] - search.points[index][position.dimension];
        let left = TreePosition {
            node: position.node * 2 + 1,
            dimension: (position.dimension + 1) % DIMENSIONS,
        };
        let right = TreePosition {
            node: left.node + 1,
            dimension: left.dimension,
        };
        if plane * plane < search.distance + EPSILON {
            self.visit_recursive(search, left);
            self.visit_recursive(search, right);
        } else if plane > 0.0 {
            self.visit_recursive(search, right);
        } else {
            self.visit_recursive(search, left);
        }
    }
}
