use super::Point;
use kd_tree::KdTree;
use priority_queue::EndPointHeap;

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

    #[cfg(test)]
    pub(in crate::geometry) fn for_queue_test(distance_out: f64) -> Self {
        Self {
            position: [0.0, 0.0],
            chain_id: 0,
            edge_out: NO_INDEX,
            distance_out,
            heap_index: NO_INDEX,
        }
    }

    #[cfg(test)]
    pub(in crate::geometry) fn heap_index_for_test(&self) -> usize {
        self.heap_index
    }

    #[cfg(test)]
    pub(in crate::geometry) fn set_distance_for_test(&mut self, distance_out: f64) {
        self.distance_out = distance_out;
    }
}

pub(crate) fn chain_points(points: &[Point]) -> Vec<usize> {
    if points.len() < 2 {
        return (0..points.len()).collect();
    }

    let mut end_points = Vec::with_capacity(points.len() * 2);
    for &point in points {
        end_points.push(EndPoint::new(point));
        end_points.push(EndPoint::new(point));
    }
    let positions = end_points
        .iter()
        .map(|end_point| end_point.position)
        .collect::<Vec<_>>();
    let tree = KdTree::new(&positions);

    for index in 0..end_points.len() {
        let closest = tree
            .closest(&positions, end_points[index].position, |candidate| {
                (candidate ^ index) > 1
            })
            .expect("each point must have an endpoint from another segment");
        end_points[index].edge_out = closest;
        end_points[index].distance_out =
            squared_distance(end_points[index].position, end_points[closest].position);
    }

    let mut queue = EndPointHeap::with_capacity(end_points.len());
    for index in 0..end_points.len() {
        queue.push(index, &mut end_points);
    }
    let mut chains = EquivalentChains::new(points.len());
    let mut remaining_connections = points.len() - 1;

    let first = loop {
        let point1 = queue.top();
        let point2 = end_points[point1].edge_out;
        let chain1 = chains.find(end_points[point1 ^ 1].chain_id);
        let chain2 = chains.find(end_points[point2 ^ 1].chain_id);
        let valid = end_points[point2].chain_id == 0 && (chain1 == 0 || chain1 != chain2);

        if valid {
            queue.pop(&mut end_points);
            let point2_heap_index = end_points[point2].heap_index;
            queue.remove(point2_heap_index, &mut end_points);
            end_points[point1].edge_out = point2;
            end_points[point2].edge_out = point1;
            end_points[point2].distance_out = end_points[point1].distance_out;

            let chain = match (chain1, chain2) {
                (0, 0) => chains.next(),
                (0, chain) | (chain, 0) => chain,
                (left, right) if left == right => left,
                (left, right) => chains.merge(left, right),
            };
            end_points[point1].chain_id = chain;
            end_points[point2].chain_id = chain;

            remaining_connections -= 1;
            if remaining_connections == 0 {
                let first = queue.top();
                queue.pop(&mut end_points);
                end_points[first].edge_out = NO_INDEX;
                let last = queue.top();
                end_points[last].edge_out = NO_INDEX;
                queue.pop(&mut end_points);
                break first;
            }
        } else {
            let closest = tree
                .closest(&positions, end_points[point1].position, |candidate| {
                    (candidate ^ point1) > 1
                        && end_points[candidate].chain_id == 0
                        && chains.can_connect(
                            end_points[point1 ^ 1].chain_id,
                            end_points[candidate ^ 1].chain_id,
                        )
                })
                .expect("an open chain endpoint must have a valid peer");
            end_points[point1].edge_out = closest;
            end_points[point1].distance_out =
                squared_distance(end_points[point1].position, end_points[closest].position);
            let heap_index = end_points[point1].heap_index;
            queue.update(heap_index, &mut end_points);
        }
    };

    let mut order = Vec::with_capacity(points.len());
    let mut current = first;
    while current != NO_INDEX {
        order.push(current >> 1);
        current = end_points[current ^ 1].edge_out;
    }
    order
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
