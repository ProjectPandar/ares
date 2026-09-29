//! Static `contours_simplified` (`SupportMaterial.cpp:204-330`).

use crate::geometry::{Coord, Point, Polygon};

/// `contours_simplified` (`SupportMaterial.cpp:204-330`, the static AGG-raster
/// version): walk the byte grid collecting boundary line segments, chain them
/// into contours, scale back to world coords, shrink corners by `offset`, and
/// drop collinear points. `fill_holes` runs the left/right + top/bottom
/// empty-cell fill (`:210-222`).
pub(crate) fn contours_simplified(
    grid_w: usize,
    grid_h: usize,
    pixel_size: f64,
    left_bottom: Point,
    grid: &[u8],
    offset: Coord,
    fill_holes: bool,
) -> Vec<Polygon> {
    // `cell_inside`: optionally hole-filled copy of the grid.
    let filled: Vec<u8>;
    let cell_inside: &[u8] = if fill_holes {
        filled = fill_grid_holes(grid, grid_w, grid_h);
        &filled
    } else {
        grid
    };

    // 1) Collect boundary lines.
    let mut lines: Vec<((i64, i64), (i64, i64))> = Vec::new();
    // (start_point, line_index) sorted lookup.
    let mut start_to_idx: Vec<((i64, i64), usize)> = Vec::new();
    for r in 1..grid_h {
        for c in 1..grid_w {
            let addr = r * grid_w + c;
            let left = cell_inside[addr - 1] != 0;
            let top = cell_inside[addr - grid_w] != 0;
            let current = cell_inside[addr] != 0;
            if left != current {
                let (a, b) = if left {
                    ((c as i64, (r + 1) as i64), (c as i64, r as i64))
                } else {
                    ((c as i64, r as i64), (c as i64, (r + 1) as i64))
                };
                lines.push((a, b));
                start_to_idx.push((a, lines.len() - 1));
            }
            if top != current {
                let (a, b) = if top {
                    ((c as i64, r as i64), ((c + 1) as i64, r as i64))
                } else {
                    (((c + 1) as i64, r as i64), (c as i64, r as i64))
                };
                lines.push((a, b));
                start_to_idx.push((a, lines.len() - 1));
            }
        }
    }
    start_to_idx.sort_by_key(|&(p, _)| p);

    // 2) Chain the lines into contours.
    let mut processed = vec![false; lines.len()];
    let mut out: Vec<Vec<(i64, i64)>> = Vec::new();
    for i_candidate in 0..lines.len() {
        if processed[i_candidate] {
            continue;
        }
        let mut poly_pts: Vec<(i64, i64)> = Vec::new();
        processed[i_candidate] = true;
        poly_pts.push(lines[i_candidate].1);
        let mut i_current = i_candidate;
        loop {
            // equal_range on start_to_idx for lines[i_current].b.
            let target = lines[i_current].1;
            let lo = start_to_idx.partition_point(|&(p, _)| p < target);
            let hi = start_to_idx.partition_point(|&(p, _)| p <= target);
            let mut i_next: i64 = -1;
            let mut closed = false;
            for entry in &start_to_idx[lo..hi] {
                let idx = entry.1;
                if idx == i_candidate {
                    closed = true;
                    break;
                }
                if processed[idx] {
                    continue;
                }
                if i_next == -1 {
                    i_next = idx as i64;
                } else {
                    // Corner: pick the line enclosing the smallest angle.
                    let cur = lines[i_current];
                    let nxt = lines[idx];
                    let v1 = (cur.1.0 - cur.0.0, cur.1.1 - cur.0.1);
                    let v2 = (nxt.1.0 - nxt.0.0, nxt.1.1 - nxt.0.1);
                    let cross = v1.0 * v2.1 - v2.0 * v1.1;
                    if cross > 0 {
                        i_next = idx as i64;
                        break;
                    }
                }
            }
            if closed {
                break;
            }
            if i_next < 0 {
                break;
            }
            processed[i_next as usize] = true;
            i_current = i_next as usize;
            poly_pts.push(lines[i_current].1);
        }
        out.push(poly_pts);
    }

    // 3) Scale back to world, shrink corners by `offset`, drop collinear.
    let mut result = Vec::new();
    for grid_pts in out {
        let pts: Vec<Point> = grid_pts
            .iter()
            .map(|&(gx, gy)| {
                Point::new(
                    (gx as f64 * pixel_size) as Coord + left_bottom.x(),
                    (gy as f64 * pixel_size) as Coord + left_bottom.y(),
                )
            })
            .collect();
        result.push(Polygon::new(shrink_corners(&pts, offset)));
    }
    result
}

/// `cell_inside_data` fill (`:211-222`): mark an empty cell if both its
/// left/right OR top/bottom neighbours are filled.
fn fill_grid_holes(grid: &[u8], grid_w: usize, grid_h: usize) -> Vec<u8> {
    let mut data = grid.to_vec();
    for r in 1..grid_h - 1 {
        for c in 1..grid_w - 1 {
            let addr = r * grid_w + c;
            if (grid[addr - 1] != 0 && grid[addr + 1] != 0)
                || (grid[addr - grid_w] != 0 && grid[addr + grid_w] != 0)
            {
                data[addr] = 1;
            }
        }
    }
    data
}

/// Corner shrink + collinear drop (`:309-326`): keep only corner points
/// (both components of the span vector non-zero), shifted by `offset`.
fn shrink_corners(pts: &[Point], offset: Coord) -> Vec<Point> {
    let mut out = Vec::with_capacity(pts.len());
    let n = pts.len();
    for j in 0..n {
        let j0 = if j == 0 { n - 1 } else { j - 1 };
        let j2 = if j + 1 == n { 0 } else { j + 1 };
        let vx = pts[j2].x() - pts[j0].x();
        let vy = pts[j2].y() - pts[j0].y();
        if vx != 0 && vy != 0 {
            let mut p = pts[j];
            let ny = p.y() + if vx < 0 { -offset } else { offset };
            let nx = p.x() + if vy > 0 { -offset } else { offset };
            p = Point::new(nx, ny);
            out.push(p);
        }
    }
    out
}
