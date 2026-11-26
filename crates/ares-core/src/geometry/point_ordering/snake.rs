//! OrderingStrategies.cpp:282–438 (`snake_core` and row-serpentine helpers) at
//! OrcaSlicer 40ecc66f78ed72dc368e0c1ebe40d77f00cc8a19.

use std::collections::BTreeMap;

use super::{Point, crossings, position, squared, two_opt};

const MIN_GAP_FILTER: f64 = 1.0;
const FRACTION_OF_Y_RANGE: f64 = 0.02;
const MIN_THRESHOLD_UM: f64 = 1e4;

struct SnakeRow {
    avg_y: f64,
    indices: Vec<usize>,
}

pub(super) fn snake_core(centers: &[Point]) -> Vec<usize> {
    if centers.is_empty() {
        return Vec::new();
    }
    let mut path = row_serpentine_path(centers);
    for _ in 0..3 {
        let mut improved = two_opt::improve(&mut path, centers);
        improved |= crossings::remove(&mut path, centers);
        if !improved {
            break;
        }
    }
    path
}

fn row_serpentine_path(centers: &[Point]) -> Vec<usize> {
    if centers.is_empty() {
        return Vec::new();
    }
    let mut sorted_ys = centers
        .iter()
        .map(|point| point.y() as f64)
        .collect::<Vec<_>>();
    sorted_ys.sort_by(f64::total_cmp);
    let y_min = sorted_ys[0];
    let y_max = sorted_ys[sorted_ys.len() - 1];
    let row_threshold = compute_row_threshold(&sorted_ys, y_min, y_max, centers.len());
    let mut rows = group_into_rows(centers, row_threshold);
    build_serpentine_path(centers, &mut rows)
}

fn compute_row_threshold(sorted_ys: &[f64], y_min: f64, y_max: f64, count: usize) -> f64 {
    let mut unique_ys = Vec::with_capacity(sorted_ys.len());
    unique_ys.push(sorted_ys[0]);
    for index in 1..sorted_ys.len() {
        if sorted_ys[index] - sorted_ys[index - 1] > MIN_GAP_FILTER {
            unique_ys.push(sorted_ys[index]);
        }
    }
    let fallback_threshold = (y_max - y_min) * FRACTION_OF_Y_RANGE;
    if unique_ys.len() <= 1 {
        return fallback_threshold.max(MIN_THRESHOLD_UM);
    }
    let mut gaps = unique_ys
        .windows(2)
        .map(|pair| pair[1] - pair[0])
        .collect::<Vec<_>>();
    if gaps.is_empty() {
        return fallback_threshold.max(MIN_THRESHOLD_UM);
    }
    gaps.sort_by(f64::total_cmp);
    let median_gap = gaps[gaps.len() / 2];
    let min_gap = gaps[0];
    let threshold = if median_gap < min_gap * 1.5 {
        min_gap * 0.5
    } else {
        median_gap * 0.5
    };
    let has_row_structure = if unique_ys.len() * 2 <= count {
        true
    } else {
        let max_gap = gaps[gaps.len() - 1];
        max_gap < min_gap * 2.0
    };
    if has_row_structure {
        return threshold;
    }
    fallback_threshold.max(MIN_THRESHOLD_UM)
}

fn group_into_rows(centers: &[Point], row_threshold: f64) -> Vec<SnakeRow> {
    let mut row_map = BTreeMap::<i64, Vec<usize>>::new();
    for (index, point) in centers.iter().enumerate() {
        let y_key = (point.y() as f64 / row_threshold).floor() as i64;
        row_map.entry(y_key).or_default().push(index);
    }
    let mut rows = Vec::with_capacity(row_map.len());
    for (_, indices) in row_map {
        let sum_y = indices
            .iter()
            .fold(0.0_f64, |acc, &index| acc + centers[index].y() as f64);
        let avg_y = sum_y / indices.len() as f64;
        rows.push(SnakeRow { avg_y, indices });
    }
    rows.sort_by(|a, b| a.avg_y.total_cmp(&b.avg_y));
    rows
}

fn build_serpentine_path(centers: &[Point], rows: &mut [SnakeRow]) -> Vec<usize> {
    let mut path = Vec::with_capacity(centers.len());
    for (row_index, row) in rows.iter_mut().enumerate() {
        crate::geometry::fixed_gcc_sort_by(&mut row.indices, |&a, &b| {
            centers[a].x() < centers[b].x()
        });
        if row_index == 0 {
            path.extend_from_slice(&row.indices);
            continue;
        }
        let prev_end = position(centers[*path.last().expect("previous row emitted at least one")]);
        let left = position(centers[row.indices[0]]);
        let right = position(centers[*row.indices.last().expect("row is non-empty")]);
        if squared(prev_end, left) <= squared(prev_end, right) {
            path.extend_from_slice(&row.indices);
        } else {
            path.extend(row.indices.iter().rev().copied());
        }
    }
    path
}
