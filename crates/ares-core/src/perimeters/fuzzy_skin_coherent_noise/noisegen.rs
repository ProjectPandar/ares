//! `noisegen.cpp` + module `GetValue` ports (Orca-deps-libnoise 1.0):
//! `GradientCoherentNoise3D`, `GradientNoise3D`, `IntValueNoise3D`,
//! `ValueNoise3D`, and the Perlin/Billow/RidgedMulti/Voronoi octave loops.
//!
//! `NOISE_VERSION == 2` constants (X/Y/Z/SEED_NOISE_GEN, SHIFT=8) are the
//! ones upstream builds with.

use super::vectortable::RANDOM_VECTORS;
use super::{lattice_index, lerp, make_int32_range, scurve3};

const X_NOISE_GEN: i32 = 1619;
const Y_NOISE_GEN: i32 = 31337;
const Z_NOISE_GEN: i32 = 6971;
const SEED_NOISE_GEN: i32 = 1013;
const SHIFT_NOISE_GEN: i32 = 8;

/// `noisegen.cpp:GradientNoise3D` — `QUALITY_STD` path is inlined by the
/// caller through `gradient_coherent_noise3d` below.
fn gradient_noise_3d([fx, fy, fz]: [f64; 3], [ix, iy, iz]: [i32; 3], seed: i32) -> f64 {
    // Upstream arithmetic is `int` (32-bit, wrapping) then masked to 0xff.
    let mut vector_index = (X_NOISE_GEN
        .wrapping_mul(ix)
        .wrapping_add(Y_NOISE_GEN.wrapping_mul(iy))
        .wrapping_add(Z_NOISE_GEN.wrapping_mul(iz))
        .wrapping_add(SEED_NOISE_GEN.wrapping_mul(seed))) as u32;
    vector_index ^= vector_index >> SHIFT_NOISE_GEN;
    vector_index &= 0xff;
    // `g_randomVectors[vectorIndex << 2]` — the table is stored as 256
    // `f64x4` rows, so the row index is `vectorIndex` directly.
    let vi = vector_index as usize;
    let xv_gradient = RANDOM_VECTORS[vi][0];
    let yv_gradient = RANDOM_VECTORS[vi][1];
    let zv_gradient = RANDOM_VECTORS[vi][2];
    let xv_point = fx - ix as f64;
    let yv_point = fy - iy as f64;
    let zv_point = fz - iz as f64;
    (xv_gradient * xv_point + yv_gradient * yv_point + zv_gradient * zv_point) * 2.12
}

/// `noisegen.cpp:GradientCoherentNoise3D` with `noiseQuality ==
/// QUALITY_STD` (the default every fuzzy-skin module uses).
fn gradient_coherent_noise_3d(x: f64, y: f64, z: f64, seed: i32) -> f64 {
    let x0 = lattice_index(x);
    let x1 = x0 + 1;
    let y0 = lattice_index(y);
    let y1 = y0 + 1;
    let z0 = lattice_index(z);
    let z1 = z0 + 1;

    let xs = scurve3(x - x0 as f64);
    let ys = scurve3(y - y0 as f64);
    let zs = scurve3(z - z0 as f64);

    let mut n0 = gradient_noise_3d([x, y, z], [x0, y0, z0], seed);
    let mut n1 = gradient_noise_3d([x, y, z], [x1, y0, z0], seed);
    let ix0 = lerp(n0, n1, xs);
    n0 = gradient_noise_3d([x, y, z], [x0, y1, z0], seed);
    n1 = gradient_noise_3d([x, y, z], [x1, y1, z0], seed);
    let ix1 = lerp(n0, n1, xs);
    let iy0 = lerp(ix0, ix1, ys);

    n0 = gradient_noise_3d([x, y, z], [x0, y0, z1], seed);
    n1 = gradient_noise_3d([x, y, z], [x1, y0, z1], seed);
    let ix0 = lerp(n0, n1, xs);
    n0 = gradient_noise_3d([x, y, z], [x0, y1, z1], seed);
    n1 = gradient_noise_3d([x, y, z], [x1, y1, z1], seed);
    let ix1 = lerp(n0, n1, xs);
    let iy1 = lerp(ix0, ix1, ys);

    lerp(iy0, iy1, zs)
}

/// `noisegen.cpp:IntValueNoise3D` — 32-bit `int` wrapping arithmetic.
fn int_value_noise_3d(x: i32, y: i32, z: i32, seed: i32) -> i32 {
    let mut n = (X_NOISE_GEN
        .wrapping_mul(x)
        .wrapping_add(Y_NOISE_GEN.wrapping_mul(y))
        .wrapping_add(Z_NOISE_GEN.wrapping_mul(z))
        .wrapping_add(SEED_NOISE_GEN.wrapping_mul(seed)))
        & 0x7fff_ffff;
    n = (n >> 13) ^ n;
    n.wrapping_mul(
        n.wrapping_mul(n)
            .wrapping_mul(60493)
            .wrapping_add(19_990_303),
    )
    .wrapping_add(1_376_312_589)
        & 0x7fff_ffff
}

/// `noisegen.cpp:ValueNoise3D`.
fn value_noise_3d(x: i32, y: i32, z: i32, seed: i32) -> f64 {
    1.0 - f64::from(int_value_noise_3d(x, y, z, seed)) / 1_073_741_824.0
}

/// `module/perlin.cpp::Perlin::GetValue` — defaults: lacunarity 2.0,
/// `QUALITY_STD`, seed 0 (`perlin.h:44-59`).
pub(super) fn perlin(
    [mut x, mut y, mut z]: [f64; 3],
    frequency: f64,
    octave_count: usize,
    persistence: f64,
) -> f64 {
    x *= frequency;
    y *= frequency;
    z *= frequency;
    let mut value = 0.0;
    let mut cur_persistence = 1.0;
    for cur_octave in 0..octave_count {
        let nx = make_int32_range(x);
        let ny = make_int32_range(y);
        let nz = make_int32_range(z);
        let seed = (0i32.wrapping_add(cur_octave as i32)) as u32;
        let signal = gradient_coherent_noise_3d(nx, ny, nz, seed as i32);
        value += signal * cur_persistence;
        x *= 2.0;
        y *= 2.0;
        z *= 2.0;
        cur_persistence *= persistence;
    }
    value
}

/// `module/billow.cpp::Billow::GetValue` — same octave loop as Perlin with
/// `2*|signal| - 1` shaping and the `+0.5` tail bias.
pub(super) fn billow(
    [mut x, mut y, mut z]: [f64; 3],
    frequency: f64,
    octave_count: usize,
    persistence: f64,
) -> f64 {
    x *= frequency;
    y *= frequency;
    z *= frequency;
    let mut value = 0.0;
    let mut cur_persistence = 1.0;
    for cur_octave in 0..octave_count {
        let nx = make_int32_range(x);
        let ny = make_int32_range(y);
        let nz = make_int32_range(z);
        let seed = cur_octave as i32;
        let mut signal = gradient_coherent_noise_3d(nx, ny, nz, seed);
        signal = 2.0 * signal.abs() - 1.0;
        value += signal * cur_persistence;
        x *= 2.0;
        y *= 2.0;
        z *= 2.0;
        cur_persistence *= persistence;
    }
    value + 0.5
}

/// `module/ridgedmulti.cpp::RidgedMulti::GetValue` — spectral weights from
/// `CalcSpectralWeights` (lacunarity 2.0, h=1.0 → `pow(2^-octave)`), seed
/// masked `& 0x7fffffff`, gain 2.0, offset 1.0, `value*1.25 - 1.0` tail.
pub(super) fn ridged_multi(
    mut x: f64,
    mut y: f64,
    mut z: f64,
    frequency: f64,
    octave_count: usize,
) -> f64 {
    // `CalcSpectralWeights`: `pow(frequency, -h)` with h=1, lacunarity 2.
    let mut spectral_weights = [0.0f64; 30];
    let mut freq = 1.0f64;
    for w in &mut spectral_weights {
        *w = freq.powf(-1.0);
        freq *= 2.0;
    }
    x *= frequency;
    y *= frequency;
    z *= frequency;
    let mut value = 0.0;
    let mut weight = 1.0f64;
    let offset = 1.0;
    let gain = 2.0;
    for (cur_octave, &spectral_weight) in spectral_weights.iter().enumerate().take(octave_count) {
        let nx = make_int32_range(x);
        let ny = make_int32_range(y);
        let nz = make_int32_range(z);
        let seed = cur_octave as i32 & 0x7fff_ffff;
        let mut signal = gradient_coherent_noise_3d(nx, ny, nz, seed);
        signal = signal.abs();
        signal = offset - signal;
        signal *= signal;
        signal *= weight;
        weight = signal * gain;
        weight = weight.clamp(0.0, 1.0);
        value += signal * spectral_weight;
        x *= 2.0;
        y *= 2.0;
        z *= 2.0;
    }
    value * 1.25 - 1.0
}

/// `module/voronoi.cpp::Voronoi::GetValue` — `m_enableDistance == false`
/// path (distance folded value is 0), feature points from three
/// `ValueNoise3D` seeds, displacement applied to the winner's value noise.
pub(super) fn voronoi(
    mut x: f64,
    mut y: f64,
    mut z: f64,
    frequency: f64,
    displacement: f64,
) -> f64 {
    x *= frequency;
    y *= frequency;
    z *= frequency;
    let x_int = lattice_index(x);
    let y_int = lattice_index(y);
    let z_int = lattice_index(z);
    let mut min_dist = 2_147_483_647.0f64;
    let mut x_candidate = 0.0;
    let mut y_candidate = 0.0;
    let mut z_candidate = 0.0;
    for (z_cur, y_cur) in ((z_int - 2)..=(z_int + 2))
        .flat_map(|z_cur| ((y_int - 2)..=(y_int + 2)).map(move |y_cur| (z_cur, y_cur)))
    {
        for x_cur in (x_int - 2)..=(x_int + 2) {
            let x_pos = x_cur as f64 + value_noise_3d(x_cur, y_cur, z_cur, 0);
            let y_pos = y_cur as f64 + value_noise_3d(x_cur, y_cur, z_cur, 1);
            let z_pos = z_cur as f64 + value_noise_3d(x_cur, y_cur, z_cur, 2);
            let x_dist = x_pos - x;
            let y_dist = y_pos - y;
            let z_dist = z_pos - z;
            let dist = x_dist * x_dist + y_dist * y_dist + z_dist * z_dist;
            if dist < min_dist {
                min_dist = dist;
                x_candidate = x_pos;
                y_candidate = y_pos;
                z_candidate = z_pos;
            }
        }
    }
    // `m_enableDistance == false` → value = 0.
    displacement
        * value_noise_3d(
            x_candidate.floor() as i32,
            y_candidate.floor() as i32,
            z_candidate.floor() as i32,
            0,
        )
}
