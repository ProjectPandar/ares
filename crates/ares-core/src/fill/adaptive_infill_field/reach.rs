//! Spherical/circular lobe reach raymarching, direction smoothing, and
//! `DistanceWarp` mean-depth profiling (`AdaptiveInfillField.cpp:351-449`).

use crate::ProcessAdaptiveInfillMode;

use super::{
    AZIMUTH, AdaptiveInfillField, Lobe, POLAR,
    grid::{DistanceGrid, node_of, norm3},
};

const LOBE_REACH: f64 = 2.0;
const SAMPLES: usize = 32;
const PI: f64 = std::f64::consts::PI;

struct RayContext<'a> {
    grid: &'a DistanceGrid,
    body_map: &'a [i32],
    body_id: i32,
    strides: (usize, usize),
    others: &'a [(f64, usize)],
    centers_depths: &'a [([f64; 3], f64)],
}

pub(super) fn compute_lobe_reaches(
    field: &AdaptiveInfillField,
    grid: &DistanceGrid,
    body_map: &[i32],
    lobes: &mut [Lobe],
) {
    if lobes.is_empty() {
        return;
    }
    let sy = grid.size[0] as usize;
    let sz = sy * grid.size[1] as usize;
    let mut lobe_body = vec![0_i32; lobes.len()];
    for (id, b) in field.bodies.iter().enumerate() {
        lobe_body[b.first_lobe..b.first_lobe + b.lobes].fill(id as i32);
    }
    let dirs = field.directions();
    let rows = dirs / AZIMUTH;
    let centers_depths: Vec<([f64; 3], f64)> = lobes.iter().map(|l| (l.center, l.depth)).collect();

    for l in 0..lobes.len() {
        let body_id = lobe_body[l];
        let others = sorted_other_lobes(
            field.bodies[body_id as usize].first_lobe,
            field.bodies[body_id as usize].lobes,
            l,
            &centers_depths,
        );
        let ctx = RayContext {
            grid,
            body_map,
            body_id,
            strides: (sy, sz),
            others: &others,
            centers_depths: &centers_depths,
        };
        let mut log_reach = raymarch_lobe(&ctx, field.axis, rows, centers_depths[l]);
        smooth_directions(&mut log_reach, rows, 1);
        lobes[l].reach = log_reach.iter().map(|&v| v.exp() as f32).collect();

        if field.mode == ProcessAdaptiveInfillMode::DistanceWarp {
            lobes[l].mean_depth =
                compute_mean_depth(field, rows, centers_depths[l].0, &lobes[l].reach);
        }
    }
}

fn sorted_other_lobes(
    first_lobe: usize,
    lobe_count: usize,
    self_index: usize,
    centers_depths: &[([f64; 3], f64)],
) -> Vec<(f64, usize)> {
    let (lobe_center, lobe_depth) = centers_depths[self_index];
    let mut others = Vec::new();
    for (k, &(other_center, other_depth)) in centers_depths
        .iter()
        .enumerate()
        .take(first_lobe + lobe_count)
        .skip(first_lobe)
    {
        if k == self_index {
            continue;
        }
        let dist = norm3([
            other_center[0] - lobe_center[0],
            other_center[1] - lobe_center[1],
            other_center[2] - lobe_center[2],
        ]) / (1.0 + other_depth / lobe_depth);
        others.push((dist, k));
    }
    others.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
    others
}

fn raymarch_lobe(
    ctx: &RayContext<'_>,
    axis: i32,
    rows: usize,
    (lobe_center, lobe_depth): ([f64; 3], f64),
) -> Vec<f64> {
    let mut log_reach = vec![0.0_f64; rows * AZIMUTH];
    for i in 0..rows {
        for j in 0..AZIMUTH {
            let dir = direction_vec(axis, i, j);
            log_reach[i * AZIMUTH + j] = march_ray(ctx, lobe_center, lobe_depth, dir).ln();
        }
    }
    log_reach
}

fn march_ray(ctx: &RayContext<'_>, center: [f64; 3], lobe_depth: f64, dir: [f64; 3]) -> f64 {
    let step = 0.5 * ctx.grid.cell;
    let mut r = 0.0_f64;
    let mut limit = f64::INFINITY;
    loop {
        let next_r = r + step;
        let pt = [
            center[0] + next_r * dir[0],
            center[1] + next_r * dir[1],
            center[2] + next_r * dir[2],
        ];
        let Some(n) = node_of(ctx.grid, pt, ctx.strides) else {
            break;
        };
        if ctx.grid.depth[n] == 0.0 || ctx.body_map[n] != ctx.body_id || next_r >= limit {
            break;
        }
        if limit == f64::INFINITY && nearer_lobe(ctx, lobe_depth, pt, next_r) {
            limit = LOBE_REACH * next_r;
        }
        r = next_r;
    }
    (r + 0.5 * step).min(limit)
}

fn compute_mean_depth(
    field: &AdaptiveInfillField,
    rows: usize,
    center: [f64; 3],
    reaches: &[f32],
) -> Vec<f32> {
    let dirs = reaches.len();
    let mut mean = vec![0.0_f64; dirs * (SAMPLES + 1)];
    let center_depth = field.depth(center);
    for (d, &reach_f32) in reaches.iter().enumerate() {
        let dir = direction_vec(field.axis, d / AZIMUTH, d % AZIMUTH);
        let row = &mut mean[d * (SAMPLES + 1)..(d + 1) * (SAMPLES + 1)];
        integrate_depth_ray(
            field,
            (center, dir),
            (center_depth, f64::from(reach_f32)),
            row,
        );
    }
    smooth_directions(&mut mean, rows, SAMPLES + 1);
    mean.into_iter().map(|v| v as f32).collect()
}

fn integrate_depth_ray(
    field: &AdaptiveInfillField,
    (center, dir): ([f64; 3], [f64; 3]),
    (center_depth, reach): (f64, f64),
    row: &mut [f64],
) {
    let mut integral = 0.0_f64;
    let mut previous = center_depth;
    row[0] = previous;
    for (k, slot) in row.iter_mut().enumerate().take(SAMPLES + 1).skip(1) {
        let t0 = (k - 1) as f64 / SAMPLES as f64;
        let t1 = k as f64 / SAMPLES as f64;
        let pt = [
            center[0] + t1 * reach * dir[0],
            center[1] + t1 * reach * dir[1],
            center[2] + t1 * reach * dir[2],
        ];
        let d1 = field.depth(pt);
        let b = (d1 - previous) * SAMPLES as f64;
        let a = previous - b * t0;
        let t0_3 = t0 * t0 * t0;
        let t1_3 = t1 * t1 * t1;
        let t0_4 = (t0 * t0) * (t0 * t0);
        let t1_4 = (t1 * t1) * (t1 * t1);
        integral += a * (t1_3 - t0_3) / 3.0 + b * (t1_4 - t0_4) / 4.0;
        previous = d1;
        *slot = 3.0 * integral / t1_3;
    }
}

fn nearer_lobe(ctx: &RayContext<'_>, lobe_depth: f64, pt: [f64; 3], r: f64) -> bool {
    for &(threshold, k) in ctx.others {
        if threshold >= r {
            break;
        }
        let (other_center, other_depth) = ctx.centers_depths[k];
        let dist = norm3([
            pt[0] - other_center[0],
            pt[1] - other_center[1],
            pt[2] - other_center[2],
        ]);
        if dist / other_depth < r / lobe_depth {
            return true;
        }
    }
    false
}

fn direction_vec(axis: i32, i: usize, j: usize) -> [f64; 3] {
    let azimuth = j as f64 * 2.0 * PI / AZIMUTH as f64;
    if axis < 0 {
        let polar = (i as f64 + 0.5) * PI / POLAR as f64;
        [
            polar.sin() * azimuth.cos(),
            polar.sin() * azimuth.sin(),
            polar.cos(),
        ]
    } else {
        let mut dir = [0.0; 3];
        let a = axis as usize;
        dir[(a + 1) % 3] = azimuth.cos();
        dir[(a + 2) % 3] = azimuth.sin();
        dir
    }
}

fn smooth_directions(values: &mut Vec<f64>, rows: usize, count: usize) {
    for _ in 0..2 {
        let mut smoothed = vec![0.0_f64; values.len()];
        for i in 0..rows {
            for j in 0..AZIMUTH {
                smooth_direction_cell(values, &mut smoothed, (rows, count), (i, j));
            }
        }
        *values = smoothed;
    }
}

fn smooth_direction_cell(
    values: &[f64],
    smoothed: &mut [f64],
    (rows, count): (usize, usize),
    (i, j): (usize, usize),
) {
    let dst = (i * AZIMUTH + j) * count;
    for k in 0..count {
        let mut sum = 0.0_f64;
        for di in -1_isize..=1 {
            let ri = (i as isize + di).clamp(0, rows as isize - 1) as usize;
            for dj in -1_isize..=1 {
                let cj = ((j as isize + dj + AZIMUTH as isize) as usize) % AZIMUTH;
                sum += values[(ri * AZIMUTH + cj) * count + k];
            }
        }
        smoothed[dst + k] = sum / 9.0;
    }
}
