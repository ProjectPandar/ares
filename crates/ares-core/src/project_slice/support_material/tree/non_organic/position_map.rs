//! `drop_nodes` PointHash map traversal in the selected Linux reference.

use super::nodes::NodeId;
use crate::geometry::Point;

struct Entry {
    position: Point,
    node: NodeId,
    hash: u64,
    previous: Option<usize>,
    next: Option<usize>,
}

pub(super) struct PositionMap {
    entries: Vec<Entry>,
    buckets: Vec<Option<usize>>,
    head: Option<usize>,
    growth_index: usize,
}

impl PositionMap {
    pub(super) fn new() -> Self {
        Self {
            entries: Vec::new(),
            buckets: vec![None],
            head: None,
            growth_index: 0,
        }
    }

    pub(super) fn insert(&mut self, position: Point, node: NodeId) {
        let hash = point_hash(position);
        let bucket = (hash % self.buckets.len() as u64) as usize;
        let mut current = self.buckets[bucket];
        while let Some(index) = current {
            let entry = &mut self.entries[index];
            if entry.hash % self.buckets.len() as u64 != bucket as u64 {
                break;
            }
            if entry.position == position {
                entry.node = node;
                return;
            }
            current = entry.next;
        }
        if self.entries.is_empty() || self.entries.len() + 1 > self.buckets.len() {
            self.grow();
        }
        let index = self.entries.len();
        self.entries.push(Entry {
            position,
            node,
            hash,
            previous: None,
            next: None,
        });
        self.insert_bucket_front(index);
    }

    pub(super) fn into_ordered(self) -> Vec<(Point, NodeId)> {
        let mut ordered = Vec::with_capacity(self.entries.len());
        let mut current = self.head;
        while let Some(index) = current {
            let entry = &self.entries[index];
            ordered.push((entry.position, entry.node));
            current = entry.next;
        }
        ordered
    }

    fn grow(&mut self) {
        let count = GROWTH_BUCKETS[self.growth_index] as usize;
        self.growth_index += 1;
        self.buckets = vec![None; count];
        let mut current = self.head.take();
        while let Some(index) = current {
            current = self.entries[index].next;
            self.insert_bucket_front(index);
        }
    }

    fn insert_bucket_front(&mut self, index: usize) {
        let bucket = (self.entries[index].hash % self.buckets.len() as u64) as usize;
        let first = self.buckets[bucket];
        let next = first.or(self.head);
        let previous = first.and_then(|first| self.entries[first].previous);
        self.entries[index].previous = previous;
        self.entries[index].next = next;
        if let Some(previous) = previous {
            self.entries[previous].next = Some(index);
        } else {
            self.head = Some(index);
        }
        if let Some(next) = next {
            self.entries[next].previous = Some(index);
        }
        self.buckets[bucket] = Some(index);
    }
}

fn point_hash(point: Point) -> u64 {
    point
        .x()
        .wrapping_add(89 * 31)
        .wrapping_mul(31)
        .wrapping_add(point.y()) as u64
}

// Default load1, initial13, then native _M_next_bkt(2 * previous).
const GROWTH_BUCKETS: &[u64] = &[
    13,
    29,
    59,
    127,
    257,
    541,
    1109,
    2357,
    5087,
    10273,
    20753,
    42043,
    85229,
    172933,
    351061,
    712697,
    1447153,
    2938679,
    5967347,
    12117689,
    24607243,
    49969847,
    101473717,
    206062531,
    418451333,
    849749479,
    1725587117,
    3504151727,
    8589934583,
    25769803693,
    68719476731,
    206158430123,
    412316860387,
    1099511627689,
    2199023255531,
    4398046511093,
    13194139533241,
    26388279066581,
    52776558133177,
    105553116266399,
    211106232532861,
    562949953421231,
    1125899906842597,
    4503599627370449,
    18014398509481951,
    36028797018963913,
    72057594037927931,
    288230376151711717,
    1152921504606846883,
    2305843009213693951,
    9223372036854775783,
];
