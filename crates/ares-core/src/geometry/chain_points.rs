use super::Point;

mod exact;
mod greedy;
pub(crate) use exact::chain_segments_exact;
pub(in crate::geometry) mod kd_tree;
pub(in crate::geometry) mod priority_queue;

const NO_INDEX: usize = usize::MAX;

pub(in crate::geometry) struct EndPoint {
    position: [f64; 2],
    chain_id: usize,
    edge_out: usize,
    distance_out: f64,
    heap_index: usize,
}

impl EndPoint {
    fn new(point: Point) -> Self {
        Self {
            position: [point.x() as f64, point.y() as f64],
            chain_id: 0,
            edge_out: NO_INDEX,
            distance_out: f64::MAX,
            heap_index: NO_INDEX,
        }
    }
}

pub(crate) fn chain_points(points: &[Point]) -> Vec<usize> {
    super::chain_points_with_start(points, None)
}

pub(crate) fn chain_points_with_start(points: &[Point], start_near: Option<Point>) -> Vec<usize> {
    if points.len() < 2 {
        return (0..points.len()).collect();
    }
    if points.len() <= 4 {
        let endpoints = points
            .iter()
            .flat_map(|point| {
                let position = [point.x() as f64, point.y() as f64];
                [position, position]
            })
            .collect::<Vec<_>>();
        return super::chain_segments_exact(
            &endpoints,
            None,
            start_near.map(|point| [point.x() as f64, point.y() as f64]),
        )
        .into_iter()
        .map(|(index, _)| index)
        .collect();
    }
    greedy::chain(points, start_near)
}

fn squared_distance(left: [f64; 2], right: [f64; 2]) -> f64 {
    let dx = left[0] - right[0];
    let dy = left[1] - right[1];
    dx * dx + dy * dy
}

struct EquivalentChains {
    equivalent: Vec<usize>,
}

impl EquivalentChains {
    fn new(count: usize) -> Self {
        let mut equivalent = Vec::with_capacity(count + 1);
        equivalent.push(0);
        Self { equivalent }
    }

    fn next(&mut self) -> usize {
        let chain = self.equivalent.len();
        self.equivalent.push(chain);
        chain
    }

    fn find(&mut self, mut chain: usize) -> usize {
        if chain != 0 {
            let mut last = chain;
            while self.equivalent[last] != last {
                last = self.equivalent[last];
            }
            self.equivalent[chain] = last;
            chain = last;
        }
        chain
    }

    fn merge(&mut self, left: usize, right: usize) -> usize {
        let chain = self.find(left).min(self.find(right));
        self.equivalent[left] = chain;
        self.equivalent[right] = chain;
        chain
    }

    fn can_connect(&mut self, left: usize, right: usize) -> bool {
        let left = self.find(left);
        let right = self.find(right);
        left == 0 || left != right
    }
}

const _: fn(&[Point]) -> Vec<usize> = chain_points;
