mod principal_components;

use super::{CoordinateScale, Line, Polygon};
use principal_components::{Vec2f, compute_principal_components};

// Single-insert prime growth covers all 6284 quantized angle keys.
const BUCKET_COUNTS: [usize; 10] = [13, 29, 59, 127, 257, 541, 1109, 2357, 5087, 10273];

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
            bucket_count: BUCKET_COUNTS[0],
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
            self.bucket_count =
                BUCKET_COUNTS[BUCKET_COUNTS.partition_point(|&count| count <= self.bucket_count)];
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
            self.entries.insert(0, entry);
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
        for mut group in groups.into_iter().rev() {
            group.reverse();
            self.entries.extend(group);
        }
    }
}

fn hash_double(value: f64) -> u64 {
    if value == 0.0 {
        return 0;
    }
    // libstdc++'s 64-bit double hash, independent of the Rust host word size.
    const MULTIPLIER: u64 = 0xc6a4_a793_5bd1_e995;
    let data = value.to_bits().wrapping_mul(MULTIPLIER);
    let data = (data ^ (data >> 47)).wrapping_mul(MULTIPLIER);
    let hash = (0xc70f_6907 ^ 8_u64.wrapping_mul(MULTIPLIER)) ^ data;
    let hash = hash.wrapping_mul(MULTIPLIER);
    let hash = (hash ^ (hash >> 47)).wrapping_mul(MULTIPLIER);
    hash ^ (hash >> 47)
}

fn bucket(hash: u64, bucket_count: usize) -> usize {
    (hash % bucket_count as u64) as usize
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
