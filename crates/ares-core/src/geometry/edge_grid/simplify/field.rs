//! Field construction helpers for `calculate_sdf`
//! (`OrcaSlicer/src/libslic3r/EdgeGrid.cpp:688-886`): exact segment
//! seeds with the vertex-wedge sign rule, four-pass signum
//! propagation, and the Danielsson chamfer metric propagation of the
//! unsigned iso-surface vectors.

use super::super::{EdgeGrid, GridEdge};
use crate::geometry::{Coord, Point};

pub(super) struct SeedField<'a> {
    pub(super) rows: usize,
    pub(super) cols: usize,
    pub(super) vectors: &'a mut [f32],
    pub(super) signs: &'a mut [u8],
    pub(super) distances: &'a mut [f32],
}

struct SegmentSeed {
    p1: Point,
    seg_x: Coord,
    seg_y: Coord,
    l2_seg: Coord,
    prev_x: Coord,
    prev_y: Coord,
}

/// Exact seeds around one rasterized segment
/// (`EdgeGrid.cpp:688-780`).
pub(super) fn seed_segment_corners(
    grid: &EdgeGrid,
    edge: GridEdge,
    (row, col): (usize, usize),
    output: &mut SeedField<'_>,
) {
    let (p1, p2) = grid.segment(edge);
    let (seg_x, seg_y) = (p2.x() - p1.x(), p2.y() - p1.y());
    let l2_seg = seg_x * seg_x + seg_y * seg_y;
    if l2_seg == 0 {
        return;
    }
    let contour = grid.contour(edge.contour_index);
    let p0 = contour[edge
        .segment_index
        .checked_sub(1)
        .unwrap_or(contour.len() - 1)];
    let (prev_x, prev_y) = (p1.x() - p0.x(), p1.y() - p0.y());

    let segment = SegmentSeed {
        p1,
        seg_x,
        seg_y,
        l2_seg,
        prev_x,
        prev_y,
    };
    for corner_row in row.saturating_sub(1)..row + 3 {
        if corner_row >= output.rows {
            break;
        }
        seed_corner_row(grid, &segment, (corner_row, col), output);
    }
}

fn seed_corner_row(
    grid: &EdgeGrid,
    segment: &SegmentSeed,
    (corner_row, col): (usize, usize),
    output: &mut SeedField<'_>,
) {
    let &SegmentSeed {
        p1,
        seg_x,
        seg_y,
        l2_seg,
        prev_x,
        prev_y,
    } = segment;
    for corner_col in col.saturating_sub(1)..col + 3 {
        if corner_col >= output.cols {
            break;
        }
        let addr = corner_row * output.cols + corner_col;
        let pt_x = grid.bounds_min.x() + corner_col as Coord * grid.resolution;
        let pt_y = grid.bounds_min.y() + corner_row as Coord * grid.resolution;
        let (pt_x, pt_y) = (pt_x - p1.x(), pt_y - p1.y());
        let t_pt = seg_x * pt_x + seg_y * pt_y;
        if t_pt < 0 {
            // Closest to p1: only inside the wedge of the vertex.
            let t2_pt = prev_x * pt_x + prev_y * pt_y;
            if t2_pt <= 0 {
                continue;
            }
            let dabs = ((pt_x * pt_x + pt_y * pt_y) as f64).sqrt();
            if dabs < output.distances[addr] as f64 {
                let det = prev_x * seg_y - prev_y * seg_x;
                output.distances[addr] = dabs as f32;
                output.vectors[addr * 2] = pt_x.abs() as f32;
                output.vectors[addr * 2 + 1] = pt_y.abs() as f32;
                output.signs[addr] = u8::from(det < 0) | 2;
            }
        } else if t_pt <= l2_seg {
            // Closest to the segment interior.
            let d_seg = seg_y * pt_x - seg_x * pt_y;
            let distance = d_seg as f64 / (l2_seg as f64).sqrt();
            let dabs = distance.abs();
            if dabs < output.distances[addr] as f64 {
                output.distances[addr] = dabs as f32;
                let linv = d_seg as f32 / l2_seg as f32;
                output.vectors[addr * 2] = (seg_y as f32 * linv).abs();
                output.vectors[addr * 2 + 1] = (seg_x as f32 * linv).abs();
                output.signs[addr] = u8::from(d_seg < 0) | 2;
            }
        }
        // t_pt > l2_seg: closest to p2, handled as p1 of the next
        // segment discovered in the same cell.
    }
}

/// `EdgeGrid.cpp:838-859`: four directional signum sweeps.
pub(super) fn propagate_signum(signs: &mut [u8], rows: usize, cols: usize) {
    // Top to bottom.
    for row in 0..rows {
        if row > 0 {
            for col in 0..cols {
                propagate_signum_step(signs, cols, row, col, -(cols as isize));
            }
        }
        for col in 1..cols {
            propagate_signum_step(signs, cols, row, col, -1);
        }
        for col in (0..cols.saturating_sub(1)).rev() {
            propagate_signum_step(signs, cols, row, col, 1);
        }
    }
    // Bottom to top.
    for row in (0..rows.saturating_sub(1)).rev() {
        for col in 0..cols {
            propagate_signum_step(signs, cols, row, col, cols as isize);
        }
        for col in 1..cols {
            propagate_signum_step(signs, cols, row, col, -1);
        }
        for col in (0..cols.saturating_sub(1)).rev() {
            propagate_signum_step(signs, cols, row, col, 1);
        }
    }
}

fn propagate_signum_step(signs: &mut [u8], cols: usize, row: usize, col: usize, delta: isize) {
    let addr = row * cols + col;
    if signs[addr] & 4 != 0 {
        let neighbor = (addr as isize + delta) as usize;
        if signs[neighbor] & 4 == 0 {
            signs[addr] = signs[neighbor] & 1;
        }
    }
}

/// `EdgeGrid.cpp:864-886`: Danielsson chamfer propagation of the
/// unsigned vectors (horizontal and vertical steps only; the VStep3
/// variant is commented out upstream).
pub(super) fn propagate_danielsson(
    vectors: &mut [f32],
    signs: &[u8],
    rows: usize,
    cols: usize,
    resolution: Coord,
) {
    fn step(
        vectors: &mut [f32],
        signs: &[u8],
        addr: usize,
        delta: isize,
        (inc_x, inc_y): (Coord, Coord),
    ) {
        if signs[addr] & 2 == 0 {
            let (vx, vy) = (vectors[addr * 2], vectors[addr * 2 + 1]);
            let l = vx * vx + vy * vy;
            let neighbor = (addr as isize + delta) as usize;
            let (nx, ny) = (
                vectors[neighbor * 2] + inc_x as f32,
                vectors[neighbor * 2 + 1] + inc_y as f32,
            );
            let l2 = nx * nx + ny * ny;
            if l2 < l {
                vectors[addr * 2] = nx;
                vectors[addr * 2 + 1] = ny;
            }
        }
    }
    for row in 0..rows {
        if row > 0 {
            for col in 0..cols {
                step(
                    vectors,
                    signs,
                    row * cols + col,
                    -(cols as isize),
                    (0, resolution),
                );
            }
        }
        for col in 1..cols {
            step(vectors, signs, row * cols + col, -1, (resolution, 0));
        }
        for col in (0..cols.saturating_sub(1)).rev() {
            step(vectors, signs, row * cols + col, 1, (resolution, 0));
        }
    }
    for row in (0..rows.saturating_sub(1)).rev() {
        for col in 0..cols {
            step(
                vectors,
                signs,
                row * cols + col,
                cols as isize,
                (0, resolution),
            );
        }
        for col in 1..cols {
            step(vectors, signs, row * cols + col, -1, (resolution, 0));
        }
        for col in (0..cols.saturating_sub(1)).rev() {
            step(vectors, signs, row * cols + col, 1, (resolution, 0));
        }
    }
}
