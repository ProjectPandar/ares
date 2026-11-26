//! `AdaptiveInfillField::AdaptiveInfillField` (`AdaptiveInfillField.cpp:215-350,451-489`):
//! bodies, neck-tested lobes, normalized depth, 2D section lookup, and multi-body BFS.

use std::collections::VecDeque;

use crate::{
    ProcessAdaptiveInfillMode,
    geometry::{BoundingBox, CoordinateScale},
};

use super::{
    AdaptiveInfillField, Body, Lobe, Slice,
    grid::{DistanceGrid, build_distance_grid, node_of, node_position, norm3, squared_dist},
    reach::compute_lobe_reaches,
};

const NECK_RATIO: f64 = 0.8;
const MIN_LOBE_RATIO: f64 = 0.3;

impl AdaptiveInfillField {
    pub(crate) fn new(
        slices: &[Slice<'_>],
        bbox: BoundingBox,
        mode: ProcessAdaptiveInfillMode,
        scale: CoordinateScale,
    ) -> Self {
        debug_assert!(!slices.is_empty() && mode != ProcessAdaptiveInfillMode::Disabled);
        let axis = match mode {
            ProcessAdaptiveInfillMode::NormalX => 0,
            ProcessAdaptiveInfillMode::NormalY => 1,
            ProcessAdaptiveInfillMode::NormalZ => 2,
            _ => -1,
        };
        let grid = build_distance_grid(slices, bbox, axis, scale);
        let sy = grid.size[0] as usize;
        let sz = sy * grid.size[1] as usize;
        let has_lobes = !matches!(
            mode,
            ProcessAdaptiveInfillMode::SteppedShells | ProcessAdaptiveInfillMode::SmoothBlend
        );

        let (mut body, bodies, mut lobes) =
            extract_bodies_and_lobes(&grid, axis, has_lobes, (sy, sz));
        let normalized_depth = build_normalized_depth(mode, &grid, &body, &bodies);

        let mut field = Self {
            mode,
            axis,
            origin: grid.origin,
            cell: grid.cell,
            size: grid.size,
            body: Vec::new(),
            bodies,
            lobes: Vec::new(),
            section: Vec::new(),
            depth: normalized_depth,
        };

        compute_lobe_reaches(&field, &grid, &body, &mut lobes);
        field.lobes = lobes;

        if axis >= 0 {
            field.section = build_section_lookup(&body, grid.size, axis as usize, (sy, sz));
        }

        if !has_lobes {
            return field;
        }
        if field.bodies.len() == 1 {
            body.fill(0);
            field.body = body;
            return field;
        }
        flood_nearest_body(&mut body, grid.size, axis, (sy, sz));
        field.body = body;
        field
    }
}

fn build_normalized_depth(
    mode: ProcessAdaptiveInfillMode,
    grid: &DistanceGrid,
    body: &[i32],
    bodies: &[Body],
) -> Vec<f32> {
    if !matches!(
        mode,
        ProcessAdaptiveInfillMode::SteppedShells
            | ProcessAdaptiveInfillMode::SmoothBlend
            | ProcessAdaptiveInfillMode::DistanceWarp
    ) {
        return Vec::new();
    }
    grid.depth
        .iter()
        .enumerate()
        .map(|(i, &d)| {
            if body[i] < 0 {
                return 0.0_f32;
            }
            let b = &bodies[body[i] as usize];
            (((f64::from(d).sqrt() - 0.5).max(0.0) * grid.cell / b.depth).min(1.0)) as f32
        })
        .collect()
}

fn neighbour_offsets(axis: i32, (sy, sz): (usize, usize)) -> Vec<isize> {
    (-1_isize..=1)
        .flat_map(|dz| {
            (-1_isize..=1).flat_map(move |dy| (-1_isize..=1).map(move |dx| [dx, dy, dz]))
        })
        .filter(|&d| (d[0] != 0 || d[1] != 0 || d[2] != 0) && (axis < 0 || d[axis as usize] == 0))
        .map(|d| d[2] * sz as isize + d[1] * sy as isize + d[0])
        .collect()
}

fn active_steps(axis: i32, (sy, sz): (usize, usize)) -> Vec<isize> {
    let steps = [
        1,
        -1,
        sy as isize,
        -(sy as isize),
        sz as isize,
        -(sz as isize),
    ];
    steps
        .into_iter()
        .enumerate()
        .filter_map(|(idx, step)| ((idx / 2) as i32 != axis).then_some(step))
        .collect()
}

fn flood_body(
    grid: &DistanceGrid,
    steps: &[isize],
    seed: usize,
    id: i32,
    (body, body_nodes): (&mut [i32], &mut Vec<usize>),
) -> f32 {
    body_nodes.clear();
    body_nodes.push(seed);
    body[seed] = id;
    let mut max_depth = 0.0_f32;
    let mut k = 0;
    while k < body_nodes.len() {
        let cur = body_nodes[k];
        max_depth = max_depth.max(grid.depth[cur]);
        for &step in steps {
            let j = (cur as isize + step) as usize;
            if grid.depth[j] > 0.0 && body[j] < 0 {
                body[j] = id;
                body_nodes.push(j);
            }
        }
        k += 1;
    }
    max_depth
}

fn extract_bodies_and_lobes(
    grid: &DistanceGrid,
    axis: i32,
    has_lobes: bool,
    strides: (usize, usize),
) -> (Vec<i32>, Vec<Body>, Vec<Lobe>) {
    let steps = active_steps(axis, strides);
    let neighbours = neighbour_offsets(axis, strides);
    let mut body = vec![-1_i32; grid.depth.len()];
    let mut bodies = Vec::new();
    let mut lobes = Vec::new();
    let mut body_nodes = Vec::new();

    for seed in 0..grid.depth.len() {
        if grid.depth[seed] == 0.0 || body[seed] >= 0 {
            continue;
        }
        let id = bodies.len() as i32;
        let max_depth = flood_body(grid, &steps, seed, id, (&mut body, &mut body_nodes));
        bodies.push(Body {
            first_lobe: lobes.len(),
            lobes: 0,
            depth: (f64::from(max_depth).sqrt() - 0.5) * grid.cell,
        });
        if !has_lobes {
            continue;
        }
        let added = find_body_lobes(grid, &body_nodes, &neighbours, max_depth, strides);
        bodies.last_mut().unwrap().lobes = added.len();
        lobes.extend(added);
    }
    (body, bodies, lobes)
}

fn cluster_peaks(
    grid: &DistanceGrid,
    mut peaks: Vec<usize>,
    strides: (usize, usize),
) -> Vec<Vec<usize>> {
    let mut ties = Vec::new();
    while !peaks.is_empty() {
        let front = peaks[0];
        let front_sqrt = grid.depth[front].sqrt();
        let mut tied = vec![front];
        let mut remaining = Vec::new();
        for &pk in &peaks[1..] {
            if is_necked(grid, (front, pk), strides) {
                remaining.push(pk);
            } else if grid.depth[pk].sqrt() >= front_sqrt - 1.0_f32 {
                tied.push(pk);
            }
        }
        ties.push(tied);
        peaks = remaining;
    }
    ties
}

fn find_body_lobes(
    grid: &DistanceGrid,
    body_nodes: &[usize],
    neighbours: &[isize],
    max_depth: f32,
    strides: (usize, usize),
) -> Vec<Lobe> {
    let min_peak = MIN_LOBE_RATIO * MIN_LOBE_RATIO * f64::from(max_depth);
    let mut peaks: Vec<usize> = body_nodes
        .iter()
        .copied()
        .filter(|&i| {
            let d = grid.depth[i];
            f64::from(d) >= min_peak
                && neighbours
                    .iter()
                    .all(|&n| grid.depth[(i as isize + n) as usize] <= d)
        })
        .collect();
    peaks.sort_by(|&a, &b| grid.depth[b].total_cmp(&grid.depth[a]).then(a.cmp(&b)));

    let ties = cluster_peaks(grid, peaks, strides);
    let mut result = Vec::with_capacity(ties.len());
    for tied in ties {
        result.push(lobe_from_tied(grid, &tied, strides));
    }
    result
}

fn lobe_from_tied(grid: &DistanceGrid, tied: &[usize], strides: (usize, usize)) -> Lobe {
    let mut middle = [0.0; 3];
    for &i in tied {
        let p = node_position(grid, i, strides);
        middle[0] += p[0];
        middle[1] += p[1];
        middle[2] += p[2];
    }
    let inv = 1.0 / tied.len() as f64;
    middle = [middle[0] * inv, middle[1] * inv, middle[2] * inv];
    let mut center = node_position(grid, tied[0], strides);
    let mut best_sq = squared_dist(center, middle);
    for &i in tied {
        let p = node_position(grid, i, strides);
        let sq = squared_dist(p, middle);
        if sq < best_sq {
            center = p;
            best_sq = sq;
        }
    }
    Lobe {
        center,
        depth: (f64::from(grid.depth[tied[0]]).sqrt() - 0.5) * grid.cell,
        reach: Vec::new(),
        mean_depth: Vec::new(),
    }
}

fn is_necked(grid: &DistanceGrid, (a, b): (usize, usize), strides: (usize, usize)) -> bool {
    let pa = node_position(grid, a, strides);
    let pb = node_position(grid, b, strides);
    let delta = [pb[0] - pa[0], pb[1] - pa[1], pb[2] - pa[2]];
    let limit = NECK_RATIO * NECK_RATIO * f64::from(grid.depth[a].min(grid.depth[b]));
    let samples = (norm3(delta) / (0.5 * grid.cell)).ceil() as i32;
    for s in 1..samples {
        let frac = f64::from(s) / f64::from(samples);
        let pt = [
            pa[0] + delta[0] * frac,
            pa[1] + delta[1] * frac,
            pa[2] + delta[2] * frac,
        ];
        if let Some(n) = node_of(grid, pt, strides)
            && f64::from(grid.depth[n]) < limit
        {
            return true;
        }
    }
    false
}

fn build_section_lookup(
    body: &[i32],
    size: [i32; 3],
    axis: usize,
    (sy, sz): (usize, usize),
) -> Vec<i32> {
    let n = size[axis] as usize;
    let ny = size[1] as usize;
    let mut has_body = vec![false; n];
    for (i, &b) in body.iter().enumerate() {
        if b < 0 {
            continue;
        }
        let slot = match axis {
            0 => i % sy,
            1 => (i / sy) % ny,
            _ => i / sz,
        };
        has_body[slot] = true;
    }
    (0..n).map(|k| nearest_section(&has_body, k)).collect()
}

fn nearest_section(has_body: &[bool], k: usize) -> i32 {
    for d in 0..has_body.len() {
        if k >= d && has_body[k - d] {
            return (k - d) as i32;
        }
        if k + d < has_body.len() && has_body[k + d] {
            return (k + d) as i32;
        }
    }
    -1
}

fn flood_nearest_body(body: &mut [i32], size: [i32; 3], axis: i32, (sy, sz): (usize, usize)) {
    let ny = size[1] as usize;
    let nz = size[2] as usize;
    let steps: [isize; 6] = [
        1,
        -1,
        sy as isize,
        -(sy as isize),
        sz as isize,
        -(sz as isize),
    ];
    let mut queue: VecDeque<usize> = body
        .iter()
        .enumerate()
        .filter_map(|(i, &b)| (b >= 0).then_some(i))
        .collect();
    while let Some(i) = queue.pop_front() {
        let x = i % sy;
        let y = (i / sy) % ny;
        let z = i / sz;
        let valid = [x + 1 < sy, x > 0, y + 1 < ny, y > 0, z + 1 < nz, z > 0];
        let cur = body[i];
        for (k, &step) in steps.iter().enumerate() {
            if !valid[k] || (k / 2) as i32 == axis {
                continue;
            }
            let next = (i as isize + step) as usize;
            if body[next] < 0 {
                body[next] = cur;
                queue.push_back(next);
            }
        }
    }
}
