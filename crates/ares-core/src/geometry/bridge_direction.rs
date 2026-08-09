mod principal_components;

use super::{CoordinateScale, Line, Polygon};
use principal_components::{Vec2f, compute_principal_components};

const FNV_OFFSET_BASIS: u64 = 14_695_981_039_346_656_037;
const FNV_PRIME: u64 = 1_099_511_628_211;
const MIN_BUCKETS: usize = 8;

type Vec2d = (f64, f64);

struct DirectionEntry {
    key: f64,
    normal: Vec2d,
    hash: u64,
}

struct DirectionMap {
    entries: Vec<DirectionEntry>,
    bucket_count: usize,
}

impl DirectionMap {
    fn new() -> Self {
        Self {
            entries: Vec::new(),
            bucket_count: MIN_BUCKETS,
        }
    }

    fn insert(&mut self, key: f64, normal: Vec2d) {
        let hash = hash_double(key);
        let old_bucket = bucket(hash, self.bucket_count);
        if self
            .entries
            .iter()
            .any(|entry| bucket(entry.hash, self.bucket_count) == old_bucket && entry.key == key)
        {
            return;
        }

        if self.entries.len() + 1 > self.bucket_count {
            self.bucket_count = grown_bucket_count(self.bucket_count, self.entries.len() + 1);
            self.rehash();
        }

        let new_bucket = bucket(hash, self.bucket_count);
        let entry = DirectionEntry { key, normal, hash };
        if let Some(index) = self
            .entries
            .iter()
            .position(|current| bucket(current.hash, self.bucket_count) == new_bucket)
        {
            self.entries.insert(index, entry);
        } else {
            self.entries.push(entry);
        }
    }

    fn rehash(&mut self) {
        let mut groups: Vec<Vec<DirectionEntry>> = Vec::new();
        let mut group_indices = vec![usize::MAX; self.bucket_count];
        for entry in std::mem::take(&mut self.entries) {
            let entry_bucket = bucket(entry.hash, self.bucket_count);
            let group_index = if group_indices[entry_bucket] == usize::MAX {
                let index = groups.len();
                groups.push(Vec::new());
                group_indices[entry_bucket] = index;
                index
            } else {
                group_indices[entry_bucket]
            };
            groups[group_index].push(entry);
        }
        for mut group in groups {
            group.reverse();
            self.entries.extend(group);
        }
    }
}

fn hash_double(value: f64) -> u64 {
    let value = if value == 0.0 { 0.0 } else { value };
    value
        .to_le_bytes()
        .into_iter()
        .fold(FNV_OFFSET_BASIS, |hash, byte| {
            (hash ^ u64::from(byte)).wrapping_mul(FNV_PRIME)
        })
}

fn bucket(hash: u64, bucket_count: usize) -> usize {
    hash as usize & (bucket_count - 1)
}

fn grown_bucket_count(old_bucket_count: usize, new_size: usize) -> usize {
    let required = new_size.max(MIN_BUCKETS);
    let desired = if old_bucket_count < 512 && old_bucket_count * 8 >= required {
        old_bucket_count * 8
    } else {
        required
    };
    desired.next_power_of_two()
}

fn normalized_f64(vector: Vec2d) -> Vec2d {
    let z = vector.0 * vector.0 + vector.1 * vector.1;
    if z > 0.0 {
        let norm = z.sqrt();
        (vector.0 / norm, vector.1 / norm)
    } else {
        vector
    }
}

pub(crate) fn detect_bridging_direction(
    floating_edges: &[Line],
    overhang_area: &[Polygon],
    scale: CoordinateScale,
) -> ((f64, f64), f64) {
    if floating_edges.is_empty() {
        let (_, pc2) = compute_principal_components(overhang_area, scale);
        return if pc2 == Vec2f::ZERO {
            ((1.0, 0.0), 0.0)
        } else {
            (pc2.normalized().as_f64(), 0.0)
        };
    }

    let mut directions = DirectionMap::new();
    for line in floating_edges {
        let normal = normalized_f64((
            (line.b.y() - line.a.y()) as f64,
            (-(line.b.x() - line.a.x())) as f64,
        ));
        let quantized_angle = (normal.1.atan2(normal.0) * 1000.0).ceil();
        directions.insert(quantized_angle, normal);
    }

    let mut direction_costs = directions
        .entries
        .iter()
        .map(|entry| (entry.normal, 0.0))
        .collect::<Vec<_>>();
    for line in floating_edges {
        let line_vector = (
            (line.b.x() - line.a.x()) as f64,
            (line.b.y() - line.a.y()) as f64,
        );
        for (normal, cost) in &mut direction_costs {
            *cost += (line_vector.0 * normal.0 + line_vector.1 * normal.1).abs();
        }
    }

    let mut result_direction = (1.0, 1.0);
    let mut minimum_cost = f64::MAX;
    for (normal, cost) in direction_costs {
        if cost < minimum_cost {
            result_direction = (normal.1, -normal.0);
            minimum_cost = cost;
        }
    }
    (result_direction, minimum_cost)
}
