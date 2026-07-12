use super::fuzzy_skin::{FuzzySkinConfig, FuzzySkinNoiseType};
use crate::Point2;

type Coord = (f64, f64, f64);
type Cell = (i64, i64, i64);

pub(super) fn coherent_value(
    noise_type: FuzzySkinNoiseType,
    sample: Point2,
    print_z: f64,
    config: FuzzySkinConfig,
) -> f64 {
    let frequency = 1.0 / config.scale_mm;
    let x = sample.x() * frequency;
    let y = sample.y() * frequency;
    let z = print_z * frequency;
    match noise_type {
        FuzzySkinNoiseType::Perlin => octave_noise((x, y, z), config, 0x9b3a_7c15, perlin_octave),
        FuzzySkinNoiseType::Billow => octave_noise((x, y, z), config, 0xc2b2_ae35, billow_octave),
        FuzzySkinNoiseType::RidgedMulti => ridged_multi_noise(x, y, z, config),
        FuzzySkinNoiseType::Voronoi => voronoi_noise(x, y, z),
        FuzzySkinNoiseType::Classic | FuzzySkinNoiseType::Ripple => {
            unreachable!("coherent_value only accepts coherent fuzzy skin noise types")
        }
    }
}

fn octave_noise(
    sample: Coord,
    config: FuzzySkinConfig,
    seed: u64,
    transform: fn(f64) -> f64,
) -> f64 {
    let (x, y, z) = sample;
    let mut frequency = 1.0;
    let mut amplitude = 1.0;
    let mut total = 0.0;
    let mut amplitude_sum = 0.0;
    for octave in 0..config.octaves {
        let signal = value_noise(
            x * frequency,
            y * frequency,
            z * frequency,
            seed ^ octave as u64,
        );
        total += transform(signal) * amplitude;
        amplitude_sum += amplitude;
        frequency *= 2.0;
        amplitude *= config.persistence;
    }
    clamp_unit(total / amplitude_sum)
}

fn perlin_octave(value: f64) -> f64 {
    value
}

fn billow_octave(value: f64) -> f64 {
    2.0 * value.abs() - 1.0
}

fn ridged_multi_noise(x: f64, y: f64, z: f64, config: FuzzySkinConfig) -> f64 {
    let mut frequency = 1.0;
    let mut total = 0.0;
    for octave in 0..config.octaves {
        let signal = value_noise(
            x * frequency,
            y * frequency,
            z * frequency,
            0x1656_67b1 ^ octave as u64,
        );
        total += 2.0 * (1.0 - signal.abs()).powi(2) - 1.0;
        frequency *= 2.0;
    }
    clamp_unit(total / config.octaves as f64)
}

fn voronoi_noise(x: f64, y: f64, z: f64) -> f64 {
    let base_x = x.floor() as i64;
    let base_y = y.floor() as i64;
    let base_z = z.floor() as i64;
    let mut best_distance = f64::INFINITY;
    let mut best_value = 0.0;

    for dz in -1..=1 {
        for dy in -1..=1 {
            for dx in -1..=1 {
                let cell =
                    nearest_cell_candidate((x, y, z), (base_x + dx, base_y + dy, base_z + dz));
                (best_distance, best_value) =
                    closer_voronoi_cell((best_distance, best_value), cell);
            }
        }
    }

    best_value
}

fn nearest_cell_candidate(sample: Coord, cell: Cell) -> (f64, f64) {
    let (cell_x, cell_y, cell_z) = cell;
    let feature_x = cell_x as f64 + hash_unit(cell_x, cell_y, cell_z, 0x317d_a8b7);
    let feature_y = cell_y as f64 + hash_unit(cell_x, cell_y, cell_z, 0x91e1_0da5);
    let feature_z = cell_z as f64 + hash_unit(cell_x, cell_y, cell_z, 0x6d2b_79f5);
    (
        squared_distance(sample, (feature_x, feature_y, feature_z)),
        hash_signed(cell_x, cell_y, cell_z, 0xa24b_aed4),
    )
}

fn closer_voronoi_cell(current: (f64, f64), candidate: (f64, f64)) -> (f64, f64) {
    if candidate.0 < current.0 {
        candidate
    } else {
        current
    }
}

fn value_noise(x: f64, y: f64, z: f64, seed: u64) -> f64 {
    let x0 = x.floor() as i64;
    let y0 = y.floor() as i64;
    let z0 = z.floor() as i64;
    let tx = smoothstep(x - x0 as f64);
    let ty = smoothstep(y - y0 as f64);
    let tz = smoothstep(z - z0 as f64);

    let c000 = hash_signed(x0, y0, z0, seed);
    let c100 = hash_signed(x0 + 1, y0, z0, seed);
    let c010 = hash_signed(x0, y0 + 1, z0, seed);
    let c110 = hash_signed(x0 + 1, y0 + 1, z0, seed);
    let c001 = hash_signed(x0, y0, z0 + 1, seed);
    let c101 = hash_signed(x0 + 1, y0, z0 + 1, seed);
    let c011 = hash_signed(x0, y0 + 1, z0 + 1, seed);
    let c111 = hash_signed(x0 + 1, y0 + 1, z0 + 1, seed);

    let x00 = lerp(c000, c100, tx);
    let x10 = lerp(c010, c110, tx);
    let x01 = lerp(c001, c101, tx);
    let x11 = lerp(c011, c111, tx);
    let y0 = lerp(x00, x10, ty);
    let y1 = lerp(x01, x11, ty);
    lerp(y0, y1, tz)
}

fn hash_signed(x: i64, y: i64, z: i64, seed: u64) -> f64 {
    hash_unit(x, y, z, seed) * 2.0 - 1.0
}

fn hash_unit(x: i64, y: i64, z: i64, seed: u64) -> f64 {
    let mut hash = seed ^ 0xcbf2_9ce4_8422_2325;
    for value in [x as u64, y as u64, z as u64] {
        hash ^= value.wrapping_add(0x9e37_79b9_7f4a_7c15);
        hash = hash.wrapping_mul(0x1000_0000_01b3);
        hash ^= hash >> 32;
    }
    ((hash >> 11) as f64 + 0.5) / ((1u64 << 53) as f64)
}

fn smoothstep(value: f64) -> f64 {
    value * value * (3.0 - 2.0 * value)
}

fn lerp(start: f64, end: f64, ratio: f64) -> f64 {
    start + (end - start) * ratio
}

fn squared_distance(a: Coord, b: Coord) -> f64 {
    let (ax, ay, az) = a;
    let (bx, by, bz) = b;
    let dx = ax - bx;
    let dy = ay - by;
    let dz = az - bz;
    dx * dx + dy * dy + dz * dz
}

fn clamp_unit(value: f64) -> f64 {
    value.clamp(-1.0, 1.0)
}
