//! Faithful Rust port of upstream libnoise coherent noise modules used by
//! `Feature/FuzzySkin/FuzzySkin.cpp::get_noise_module` (`:41-64`):
//! `noise::module::Perlin`, `Billow`, `RidgedMulti`, `Voronoi` from
//! `deps/libnoise` (Orca-deps-libnoise 1.0).
//!
//! Every arithmetic step mirrors the C++ exactly — including the
//! `MakeInt32Range` wrap, the `int` truncation quirks in the lattice loops,
//! the `vectorIndex` hash, and the octave accumulation order — because the
//! emitted G-code is compared byte-for-byte.

use super::fuzzy_skin::{FuzzySkinConfig, FuzzySkinNoiseType};
use crate::Point2;

mod noisegen;
mod vectortable;

/// `x0 = (x > 0.0 ? (int)x : (int)x - 1)` — upstream's lattice-floor quirk:
/// NOT a real `floor` (`x0` skips `x == 0.0` by one, and truncates
/// negatives). `noisegen.cpp:GradientCoherentNoise3D`.
fn lattice_index(x: f64) -> i32 {
    if x > 0.0 { x as i32 } else { x as i32 - 1 }
}

/// `noisegen.h::MakeInt32Range`.
fn make_int32_range(n: f64) -> f64 {
    if n >= 1_073_741_824.0 {
        2.0 * (n % 1_073_741_824.0) - 1_073_741_824.0
    } else if n <= -1_073_741_824.0 {
        2.0 * (n % 1_073_741_824.0) + 1_073_741_824.0
    } else {
        n
    }
}

fn scurve3(a: f64) -> f64 {
    a * a * (3.0 - 2.0 * a)
}

fn lerp(n0: f64, n1: f64, a: f64) -> f64 {
    (1.0 - a) * n0 + a * n1
}

pub(crate) fn coherent_value(
    noise_type: FuzzySkinNoiseType,
    sample: Point2,
    print_z: f64,
    config: FuzzySkinConfig,
) -> f64 {
    match noise_type {
        FuzzySkinNoiseType::Perlin => noisegen::perlin(
            [sample.x(), sample.y(), print_z],
            1.0 / config.scale_mm,
            config.octaves,
            config.persistence,
        ),
        FuzzySkinNoiseType::Billow => noisegen::billow(
            [sample.x(), sample.y(), print_z],
            1.0 / config.scale_mm,
            config.octaves,
            config.persistence,
        ),
        FuzzySkinNoiseType::RidgedMulti => noisegen::ridged_multi(
            sample.x(),
            sample.y(),
            print_z,
            1.0 / config.scale_mm,
            config.octaves,
        ),
        FuzzySkinNoiseType::Voronoi => {
            noisegen::voronoi(sample.x(), sample.y(), print_z, 1.0 / config.scale_mm, 1.0)
        }
        FuzzySkinNoiseType::Classic | FuzzySkinNoiseType::Ripple => {
            unreachable!("coherent_value only accepts coherent fuzzy skin noise types")
        }
    }
}
