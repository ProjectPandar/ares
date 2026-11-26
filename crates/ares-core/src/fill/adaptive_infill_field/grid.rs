//! Grid sizing, even-odd scanline rasterization, and Felzenszwalb-Huttenlocher
//! separable squared Euclidean distance transform (`AdaptiveInfillField.cpp:34-146,221-246`).

use crate::geometry::{BoundingBox, CoordinateScale, ExPolygon, Polygon};

use super::Slice;

const MAX_NODES: f64 = (1u64 << 20) as f64;
const MIN_CELL_SIZE: f64 = 0.5;

pub(super) struct DistanceGrid {
    pub(super) origin: [f64; 3],
    pub(super) cell: f64,
    pub(super) size: [i32; 3],
    pub(super) depth: Vec<f32>,
}

struct RasterLayer {
    origin: [f64; 2],
    cell: f64,
    nx: usize,
    ny: usize,
    scale: CoordinateScale,
}

pub(super) fn build_distance_grid(
    slices: &[Slice<'_>],
    bbox: BoundingBox,
    axis_mode: i32,
    scale: CoordinateScale,
) -> DistanceGrid {
    let min = [
        scale.unscale(bbox.min().x()),
        scale.unscale(bbox.min().y()),
        slices[0].bottom_z,
    ];
    let max = [
        scale.unscale(bbox.max().x()),
        scale.unscale(bbox.max().y()),
        slices[slices.len() - 1].top_z,
    ];
    let extent = [max[0] - min[0], max[1] - min[1], max[2] - min[2]];

    let mut cell = MIN_CELL_SIZE.max((extent[0] * extent[1] * extent[2] / MAX_NODES).cbrt());
    let nodes = |length: f64, c: f64| -> i32 { (length / c).ceil() as i32 + 3 };
    while f64::from(nodes(extent[0], cell))
        * f64::from(nodes(extent[1], cell))
        * f64::from(nodes(extent[2], cell))
        > MAX_NODES
    {
        cell *= 1.1;
    }
    let size = [
        nodes(extent[0], cell),
        nodes(extent[1], cell),
        nodes(extent[2], cell),
    ];
    let origin = [min[0] - cell, min[1] - cell, min[2] - cell];

    let nx = size[0] as usize;
    let ny = size[1] as usize;
    let nz = size[2] as usize;
    let sz = nx * ny;
    let mut depth = vec![0.0_f32; sz * nz];
    let layer_grid = RasterLayer {
        origin: [origin[0], origin[1]],
        cell,
        nx,
        ny,
        scale,
    };
    for k in 0..nz {
        let z = origin[2] + k as f64 * cell;
        let idx = slices.partition_point(|s| s.top_z < z);
        if idx < slices.len() && z > slices[idx].bottom_z {
            rasterize(
                slices[idx].expolygons,
                &layer_grid,
                &mut depth[k * sz..(k + 1) * sz],
            );
        }
    }
    for axis in 0..3 {
        if axis as i32 != axis_mode {
            distance_transform_axis(&mut depth, size, axis);
        }
    }
    DistanceGrid {
        origin,
        cell,
        size,
        depth,
    }
}

pub(super) fn node_position(grid: &DistanceGrid, i: usize, (sy, sz): (usize, usize)) -> [f64; 3] {
    let ny = grid.size[1] as usize;
    [
        grid.origin[0] + grid.cell * (i % sy) as f64,
        grid.origin[1] + grid.cell * ((i / sy) % ny) as f64,
        grid.origin[2] + grid.cell * (i / sz) as f64,
    ]
}

pub(super) fn node_of(
    grid: &DistanceGrid,
    pt: [f64; 3],
    (sy, sz): (usize, usize),
) -> Option<usize> {
    let x = ((pt[0] - grid.origin[0]) / grid.cell).round() as isize;
    let y = ((pt[1] - grid.origin[1]) / grid.cell).round() as isize;
    let z = ((pt[2] - grid.origin[2]) / grid.cell).round() as isize;
    if x < 0
        || y < 0
        || z < 0
        || x >= grid.size[0] as isize
        || y >= grid.size[1] as isize
        || z >= grid.size[2] as isize
    {
        return None;
    }
    Some(z as usize * sz + y as usize * sy + x as usize)
}

pub(super) fn norm3(v: [f64; 3]) -> f64 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

pub(super) fn squared_dist(a: [f64; 3], b: [f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    dx * dx + dy * dy + dz * dz
}

fn distance_transform_line(f: &[f32], d: &mut [f32], v: &mut [i32], s: &mut [f64]) {
    let mut k: isize = -1;
    for (q_idx, &fq) in f.iter().enumerate() {
        if fq == f32::INFINITY {
            continue;
        }
        let q = q_idx as i32;
        let mut x = f64::NEG_INFINITY;
        while k >= 0 {
            let vk = v[k as usize];
            x = (f64::from(fq) + f64::from(q) * f64::from(q)
                - f64::from(f[vk as usize])
                - f64::from(vk) * f64::from(vk))
                / (2.0 * f64::from(q - vk));
            if x > s[k as usize] {
                break;
            }
            k -= 1;
        }
        if k < 0 {
            x = f64::NEG_INFINITY;
        }
        k += 1;
        v[k as usize] = q;
        s[k as usize] = x;
        s[k as usize + 1] = f64::INFINITY;
    }
    if k < 0 {
        d.fill(f32::INFINITY);
        return;
    }
    let mut j = 0usize;
    for (q_idx, dq) in d.iter_mut().enumerate() {
        let q = q_idx as i32;
        while s[j + 1] < f64::from(q) {
            j += 1;
        }
        let diff = f64::from(q - v[j]);
        *dq = (diff * diff + f64::from(f[v[j] as usize])) as f32;
    }
}

fn distance_transform_axis(grid: &mut [f32], size: [i32; 3], axis: usize) {
    let n = size[axis] as usize;
    let a1 = (axis + 1) % 3;
    let a2 = (axis + 2) % 3;
    let sx = size[0] as usize;
    let sy = size[1] as usize;
    let stride = match axis {
        0 => 1,
        1 => sx,
        _ => sx * sy,
    };
    let n1 = size[a1] as usize;
    let n2 = size[a2] as usize;
    let mut f = vec![0.0_f32; n];
    let mut d = vec![0.0_f32; n];
    let mut v = vec![0_i32; n];
    let mut s = vec![0.0_f64; n + 1];
    for line in 0..(n1 * n2) {
        let mut idx = [0usize; 3];
        idx[a1] = line % n1;
        idx[a2] = line / n1;
        let first = (idx[2] * sy + idx[1]) * sx + idx[0];
        for (i, slot) in f.iter_mut().enumerate() {
            *slot = grid[first + i * stride];
        }
        distance_transform_line(&f, &mut d, &mut v, &mut s);
        for (i, &val) in d.iter().enumerate() {
            grid[first + i * stride] = val;
        }
    }
}

fn rasterize(expolygons: &[ExPolygon], grid: &RasterLayer, nodes: &mut [f32]) {
    let mut crossings = vec![Vec::<f64>::new(); grid.ny];
    let mut add_crossings = |polygon: &Polygon| {
        let pts = polygon.points();
        let len = pts.len();
        for i in 0..len {
            let pa = pts[i];
            let pb = pts[if i + 1 == len { 0 } else { i + 1 }];
            let a = [grid.scale.unscale(pa.x()), grid.scale.unscale(pa.y())];
            let b = [grid.scale.unscale(pb.x()), grid.scale.unscale(pb.y())];
            if a[1] == b[1] {
                continue;
            }
            let (lo, hi) = if a[1] <= b[1] {
                (a[1], b[1])
            } else {
                (b[1], a[1])
            };
            let j0 = (((lo - grid.origin[1]) / grid.cell).ceil() as i64).max(0) as usize;
            let j1 = (((hi - grid.origin[1]) / grid.cell).ceil() as i64).clamp(0, grid.ny as i64)
                as usize;
            for (j, row) in crossings.iter_mut().enumerate().take(j1).skip(j0) {
                let y = grid.origin[1] + j as f64 * grid.cell;
                row.push(a[0] + (b[0] - a[0]) * (y - a[1]) / (b[1] - a[1]));
            }
        }
    };
    for expolygon in expolygons {
        add_crossings(expolygon.contour());
        for hole in expolygon.holes() {
            add_crossings(hole);
        }
    }
    for (j, xs) in crossings.iter_mut().enumerate() {
        xs.sort_by(f64::total_cmp);
        let mut k = 0;
        while k + 1 < xs.len() {
            let i0 = (((xs[k] - grid.origin[0]) / grid.cell).ceil() as i64).max(0) as usize;
            let i1 = (((xs[k + 1] - grid.origin[0]) / grid.cell).ceil() as i64)
                .clamp(0, grid.nx as i64) as usize;
            let start = j * grid.nx + i0.min(i1);
            let end = j * grid.nx + i1;
            nodes[start..end].fill(f32::INFINITY);
            k += 2;
        }
    }
}
