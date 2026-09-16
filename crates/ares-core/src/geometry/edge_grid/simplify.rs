//! Signed distance field and grid-simplified contours, ported from
//! `OrcaSlicer/src/libslic3r/EdgeGrid.cpp`:
//! - `calculate_sdf` (`EdgeGrid.cpp:672-900`): exact per-corner seeds
//!   around rasterized segments (wedge sign rule at `:694-725`),
//!   two-pass signum propagation (`:838-859`), Danielsson chamfer
//!   metric propagation (`EdgeGrid.cpp:600-624`), signed assembly.
//! - `contours_simplified` (`EdgeGrid.cpp:1283-1400`): marching-squares
//!   cell classification (`cell_inside_or_crossing`,
//!   `EdgeGrid.hpp:377-388`), hole filling, line chaining with the
//!   convex-corner pick, world scaling and corner shrinking.

use std::collections::HashMap;

mod field;
use field::{propagate_danielsson, propagate_signum, seed_segment_corners};

use super::EdgeGrid;
use crate::geometry::{Coord, Point, Polygon};

pub(crate) struct SignedDistanceField {
    values: Vec<f32>,
    /// Corner columns = grid columns + 1.
    corner_cols: usize,
}

impl EdgeGrid {
    /// `EdgeGrid.cpp:672` `EdgeGrid::Grid::calculate_sdf`.
    pub(crate) fn calculate_sdf(&self) -> SignedDistanceField {
        let corner_rows = self.rows + 1;
        let corner_cols = self.cols + 1;
        let corners = corner_rows * corner_cols;
        // Unsigned vectors towards the closest point on the surface.
        let mut vectors = vec![f32::MAX; corners * 2];
        // Bit 0 negative, bit 1 original (frozen), bit 2 signum pending.
        let mut signs = vec![4u8; corners];
        let resolution = self.resolution;
        let search_radius = (resolution << 1) as f32;
        let mut field = vec![search_radius; corners];

        for row in 0..self.rows {
            for col in 0..self.cols {
                let cell = self.cells[row * self.cols + col];
                for index in cell.begin..cell.end {
                    let edge = self.cell_data[index];
                    seed_segment_corners(
                        self,
                        edge,
                        row,
                        col,
                        corner_rows,
                        corner_cols,
                        &mut vectors,
                        &mut signs,
                        &mut field,
                    );
                }
            }
        }

        propagate_signum(&mut signs, corner_rows, corner_cols);
        propagate_danielsson(&mut vectors, &signs, corner_rows, corner_cols, resolution);

        for addr in 0..corners {
            let (vx, vy) = (vectors[addr * 2], vectors[addr * 2 + 1]);
            let distance = (vx * vx + vy * vy).sqrt();
            field[addr] = if signs[addr] & 1 != 0 {
                -distance
            } else {
                distance
            };
        }
        SignedDistanceField {
            values: field,
            corner_cols,
        }
    }

    /// `EdgeGrid.hpp:377` `cell_inside_or_crossing`.
    fn cell_inside_or_crossing(&self, field: &SignedDistanceField, row: isize, col: isize) -> bool {
        if row < 0 || row as usize >= self.rows || col < 0 || col as usize >= self.cols {
            return false;
        }
        let (row, col) = (row as usize, col as usize);
        self.cell_has_edges(row, col) || field.values[row * field.corner_cols + col] <= 0.0
    }

    fn cell_has_edges(&self, row: usize, col: usize) -> bool {
        let cell = self.cells[row * self.cols + col];
        cell.begin < cell.end
    }

    /// `EdgeGrid.cpp:1283` `EdgeGrid::Grid::contours_simplified`.
    /// `offset` shrinks each corner point along the diagonal so a
    /// re-discretization is idempotent; `|2*offset| < resolution`.
    pub(crate) fn contours_simplified(
        &self,
        field: &SignedDistanceField,
        offset: Coord,
        fill_holes: bool,
    ) -> Vec<Polygon> {
        let cell_rows = self.rows + 2;
        let cell_cols = self.cols + 2;
        let mut cell_inside = vec![false; cell_rows * cell_cols];
        for row in 0..cell_rows {
            for col in 0..cell_cols {
                cell_inside[row * cell_cols + col] =
                    self.cell_inside_or_crossing(field, row as isize - 1, col as isize - 1);
            }
        }
        if fill_holes {
            let cell_inside2 = cell_inside.clone();
            for row in 1..cell_rows - 1 {
                for col in 1..cell_cols - 1 {
                    let addr = row * cell_cols + col;
                    if (cell_inside2[addr - 1] && cell_inside2[addr + 1])
                        || (cell_inside2[addr - cell_cols] && cell_inside2[addr + cell_cols])
                    {
                        cell_inside[addr] = true;
                    }
                }
            }
        }

        // Collect the marching-squares boundary lines.
        let mut lines = Vec::new();
        for row in 0..=self.rows {
            for col in 0..=self.cols {
                let addr = (row + 1) * cell_cols + col + 1;
                let left = cell_inside[addr - 1];
                let top = cell_inside[addr - cell_cols];
                let current = cell_inside[addr];
                let grid_point = |col: usize, row: usize| Point::new(col as Coord, row as Coord);
                if left != current {
                    lines.push(if left {
                        (grid_point(col, row + 1), grid_point(col, row))
                    } else {
                        (grid_point(col, row), grid_point(col, row + 1))
                    });
                }
                if top != current {
                    lines.push(if top {
                        (grid_point(col, row), grid_point(col + 1, row))
                    } else {
                        (grid_point(col + 1, row), grid_point(col, row))
                    });
                }
            }
        }

        // Chain the lines (`EdgeGrid.cpp:1342-1381`).
        let mut starts: HashMap<(Coord, Coord), Vec<usize>> = HashMap::new();
        for (index, (start, _)) in lines.iter().enumerate() {
            starts
                .entry((start.x(), start.y()))
                .or_default()
                .push(index);
        }
        let mut processed = vec![false; lines.len()];
        let mut out = Vec::new();
        for candidate in 0..lines.len() {
            if processed[candidate] {
                continue;
            }
            processed[candidate] = true;
            let mut points = vec![lines[candidate].1];
            let mut current = candidate;
            'walk: loop {
                let end = lines[current].1;
                let candidates = starts
                    .get(&(end.x(), end.y()))
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                let mut next = usize::MAX;
                for &index in candidates {
                    if index == candidate {
                        // Closing the loop.
                        break 'walk;
                    }
                    if processed[index] {
                        continue;
                    }
                    if next == usize::MAX {
                        next = index;
                    } else {
                        // Corner where two lines meet exactly: pick the
                        // one enclosing the smallest angle.
                        let (v1x, v1y) = edge_vector(&lines[current]);
                        let (v2x, v2y) = edge_vector(&lines[index]);
                        let cross =
                            i64::from(v1x) * i64::from(v2y) - i64::from(v2x) * i64::from(v1y);
                        if cross > 0 {
                            // Convex right angle; no better next line.
                            next = index;
                            break;
                        }
                    }
                }
                if next == usize::MAX {
                    break 'walk;
                }
                processed[next] = true;
                current = next;
                points.push(lines[current].1);
            }
            out.push(points);
        }

        // Scale back into world, shrink corners, drop collinear points.
        out.iter()
            .map(|points| {
                Polygon::new(
                    points
                        .iter()
                        .enumerate()
                        .filter_map(|(index, &point)| {
                            let previous = if index == 0 {
                                points.len() - 1
                            } else {
                                index - 1
                            };
                            let following = if index + 1 == points.len() {
                                0
                            } else {
                                index + 1
                            };
                            let (vx, vy) = (
                                points[following].x() - points[previous].x(),
                                points[following].y() - points[previous].y(),
                            );
                            if vx != 0 && vy != 0 {
                                let mut x = point.x() * self.resolution + self.bounds_min.x();
                                let mut y = point.y() * self.resolution + self.bounds_min.y();
                                y += if vx < 0 { -offset } else { offset };
                                x += if vy > 0 { -offset } else { offset };
                                Some(Point::new(x, y))
                            } else {
                                None
                            }
                        })
                        .collect(),
                )
            })
            .collect()
    }
}

fn edge_vector(line: &(Point, Point)) -> (Coord, Coord) {
    (line.1.x() - line.0.x(), line.1.y() - line.0.y())
}

#[cfg(test)]
mod tests;
