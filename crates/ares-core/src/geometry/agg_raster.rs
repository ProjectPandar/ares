//! Byte-exact port of the AGG `rasterizer_scanline_aa` / `rasterizer_cells_aa`
//! pipeline used by `SupportMaterial.cpp::rasterize_polygons`
//! (`SupportMaterial.cpp:169-203`): polygon vertices are upscaled to an
//! 8-bit subpixel grid, each edge is clipped into cells accumulating integer
//! `area`/`cover` (`agg_rasterizer_cells_aa.h`), and `sweep_scanline`
//! (`agg_rasterizer_scanline_aa.h:201-249`) emits a cell iff its computed
//! `alpha > 0` (even-odd fill rule). The output byte grid feeds
//! `contours_simplified`, which treats `byte != 0` as "inside".
//!
//! Only the boolean `alpha > 0` outcome is reproduced — the actual alpha
//! coverage value is not needed downstream.

use crate::geometry::{Coord, Point, Polygon};

const SUBPIXEL_SHIFT: i32 = 8;
const SUBPIXEL_SCALE: i32 = 1 << SUBPIXEL_SHIFT;
const SUBPIXEL_MASK: i32 = SUBPIXEL_SCALE - 1;
// `aa_shift = 8`, `aa_scale = 256`, `aa_mask = 255`, `aa_scale2 = 512`,
// `aa_mask2 = 511`.
const AA_SHIFT: i32 = 8;
const AA_MASK: i32 = (1 << AA_SHIFT) - 1;
const AA_SCALE: i32 = 1 << AA_SHIFT;
const AA_SCALE2: i32 = AA_SCALE * 2;

const DX_LIMIT: i64 = 16384 << SUBPIXEL_SHIFT;

/// `iround` (`agg_basics.h:131`): round-half-away-from-zero to the subpixel
/// grid. AGG's `iround(v) = floor(v + 0.5)` for the default non-sse2 path on
/// positive rasterizer coordinates; coordinates here are always >= 0 after
/// `convert_pt`, so `floor(v*scale + 0.5)` matches.
fn upscale(v: f64) -> i32 {
    (v * SUBPIXEL_SCALE as f64 + 0.5) as i32
}

#[derive(Clone, Copy, Default)]
struct Cell {
    cover: i64,
    area: i64,
}

/// Accumulate `area`/`cover` for polygon edges (a `rasterizer_cells_aa`
/// outline). Cells are keyed by `(x, y)`; insertion order is irrelevant
/// because `sweep_scanline` re-sorts per scanline.
struct Outline {
    cells: std::collections::HashMap<(i32, i32), Cell>,
    // Running cell the AGG accumulator writes into (`m_curr_cell`).
    curr: (i32, i32),
    min_y: i32,
    max_y: i32,
}

impl Outline {
    fn new() -> Self {
        Self {
            cells: std::collections::HashMap::new(),
            curr: (i32::MIN, i32::MIN),
            min_y: i32::MAX,
            max_y: i32::MIN,
        }
    }

    /// `set_curr_cell` + `add_curr_cell` (`agg_rasterizer_cells_aa.h:190-208`):
    /// flush nothing (the map already holds the running cell); just move the
    /// running coordinate.
    fn set_curr_cell(&mut self, x: i32, y: i32) {
        self.curr = (x, y);
    }

    fn cell_mut(&mut self) -> &mut Cell {
        self.cells.entry(self.curr).or_default()
    }

    /// `render_hline` (`agg_rasterizer_cells_aa.h:212-289`).
    fn render_hline(&mut self, ey: i32, x1: i32, y1: i32, x2: i32, y2: i32) {
        let mut ex1 = x1 >> SUBPIXEL_SHIFT;
        let ex2 = x2 >> SUBPIXEL_SHIFT;
        let fx1 = x1 & SUBPIXEL_MASK;
        let fx2 = x2 & SUBPIXEL_MASK;

        // Trivial case: horizontal segment inside one cell.
        if y1 == y2 {
            self.set_curr_cell(ex2, ey);
            return;
        }

        // Single cell.
        if ex1 == ex2 {
            let delta = (y2 - y1) as i64;
            let c = self.cell_mut();
            c.cover += delta;
            c.area += (fx1 + fx2) as i64 * delta;
            return;
        }

        let mut p = ((SUBPIXEL_SCALE - fx1) as i64) * ((y2 - y1) as i64);
        let mut first = SUBPIXEL_SCALE;
        let mut incr = 1i32;
        let mut dx = x2 as i64 - x1 as i64;
        if dx < 0 {
            p = (fx1 as i64) * ((y2 - y1) as i64);
            first = 0;
            incr = -1;
            dx = -dx;
        }

        let mut delta = p / dx;
        let mut m = p % dx;
        if m < 0 {
            delta -= 1;
            m += dx;
        }
        {
            let c = self.cell_mut();
            c.cover += delta;
            c.area += (fx1 + first) as i64 * delta;
        }
        ex1 += incr;
        self.set_curr_cell(ex1, ey);
        let mut y1 = y1 as i64 + delta;

        if ex1 != ex2 {
            let p = (SUBPIXEL_SCALE as i64) * ((y2 as i64) - y1 + delta);
            let lift = p / dx;
            let rem = p % dx;
            let (mut lift, mut rem) = if rem < 0 {
                (lift - 1, rem + dx)
            } else {
                (lift, rem)
            };
            m -= dx;
            while ex1 != ex2 {
                let mut delta = lift;
                m += rem;
                if m >= 0 {
                    m -= dx;
                    delta += 1;
                }
                {
                    let c = self.cell_mut();
                    c.cover += delta;
                    c.area += (SUBPIXEL_SCALE as i64) * delta;
                }
                y1 += delta;
                ex1 += incr;
                self.set_curr_cell(ex1, ey);
            }
        }
        let delta = y2 as i64 - y1;
        let c = self.cell_mut();
        c.cover += delta;
        c.area += (fx2 + SUBPIXEL_SCALE - first) as i64 * delta;
    }

    /// `line` (`agg_rasterizer_cells_aa.h:295-433`).
    fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
        let dx = x2 as i64 - x1 as i64;
        if dx >= DX_LIMIT || dx <= -DX_LIMIT {
            let cx = ((x1 as i64 + x2 as i64) >> 1) as i32;
            let cy = ((y1 as i64 + y2 as i64) >> 1) as i32;
            self.line(x1, y1, cx, cy);
            self.line(cx, cy, x2, y2);
            return;
        }

        let mut dy = y2 as i64 - y1 as i64;
        let ex1 = x1 >> SUBPIXEL_SHIFT;
        let ex2 = x2 >> SUBPIXEL_SHIFT;
        let mut ey1 = y1 >> SUBPIXEL_SHIFT;
        let ey2 = y2 >> SUBPIXEL_SHIFT;
        let fy1 = y1 & SUBPIXEL_MASK;
        let fy2 = y2 & SUBPIXEL_MASK;

        self.min_y = self.min_y.min(ey1).min(ey2);
        self.max_y = self.max_y.max(ey1).max(ey2);

        self.set_curr_cell(ex1, ey1);

        if ey1 == ey2 {
            self.render_hline(ey1, x1, fy1, x2, fy2);
            return;
        }

        // Vertical line.
        let mut incr = 1i32;
        if dx == 0 {
            let ex = x1 >> SUBPIXEL_SHIFT;
            let two_fx = ((x1 - (ex << SUBPIXEL_SHIFT)) << 1) as i64;
            let mut first = SUBPIXEL_SCALE;
            if dy < 0 {
                first = 0;
                incr = -1;
            }
            let x_from = x1;
            let delta = (first - fy1) as i64;
            {
                let c = self.cell_mut();
                c.cover += delta;
                c.area += two_fx * delta;
            }
            ey1 += incr;
            self.set_curr_cell(ex, ey1);
            let delta = (first + first - SUBPIXEL_SCALE) as i64;
            let area = two_fx * delta;
            while ey1 != ey2 {
                let c = self.cell_mut();
                c.cover = delta;
                c.area = area;
                ey1 += incr;
                self.set_curr_cell(ex, ey1);
            }
            let delta = (fy2 - SUBPIXEL_SCALE + first) as i64;
            let c = self.cell_mut();
            c.cover += delta;
            c.area += two_fx * delta;
            return;
        }

        // General: several hlines.
        let mut p = ((SUBPIXEL_SCALE - fy1) as i64) * dx;
        let mut first = SUBPIXEL_SCALE;
        if dy < 0 {
            p = (fy1 as i64) * dx;
            first = 0;
            incr = -1;
            dy = -dy;
        }
        let mut delta = p / dy;
        let mut m = p % dy;
        if m < 0 {
            delta -= 1;
            m += dy;
        }
        let mut x_from = x1 as i64 + delta;
        self.render_hline(ey1, x1, fy1, x_from as i32, first);
        ey1 += incr;
        self.set_curr_cell((x_from >> SUBPIXEL_SHIFT as i64) as i32, ey1);

        if ey1 != ey2 {
            let p = (SUBPIXEL_SCALE as i64) * dx;
            let mut lift = p / dy;
            let mut rem = p % dy;
            if rem < 0 {
                lift -= 1;
                rem += dy;
            }
            m -= dy;
            while ey1 != ey2 {
                let mut delta = lift;
                m += rem;
                if m >= 0 {
                    m -= dy;
                    delta += 1;
                }
                let x_to = x_from + delta;
                self.render_hline(
                    ey1,
                    x_from as i32,
                    SUBPIXEL_SCALE - first,
                    x_to as i32,
                    first,
                );
                x_from = x_to;
                ey1 += incr;
                self.set_curr_cell((x_from >> SUBPIXEL_SHIFT as i64) as i32, ey1);
            }
        }
        self.render_hline(ey1, x_from as i32, SUBPIXEL_SCALE - first, x2, fy2);
    }
}

/// `calculate_alpha` (`agg_rasterizer_scanline_aa.h:185-200`). Upstream
/// `rasterize_polygons` uses the default `fill_non_zero` filling rule
/// (`m_filling_rule(fill_non_zero)` at `:99`, and `rasterize_polygons` never
/// calls `filling_rule()`), so the even-odd `aa_mask2` fold is skipped —
/// `cover = abs(area >> 9)` clamped to 255. Returns 0..=255; the cell is
/// "inside" iff `alpha > 0`.
fn calculate_alpha(area: i64) -> i64 {
    let mut cover = area >> ((SUBPIXEL_SHIFT * 2 + 1) - AA_SHIFT);
    if cover < 0 {
        cover = -cover;
    }
    if cover > AA_MASK as i64 {
        cover = AA_MASK as i64;
    }
    cover
}

/// `rasterize_polygons` (`SupportMaterial.cpp:169-203`): rasterize `polygons`
/// into a `grid_w * grid_h` byte grid, cell (c,r) at `r*grid_w + c`. A cell is
/// `1` iff `sweep_scanline` emits it (`alpha > 0`). `pixel_size` is in scaled
/// Coord units; `left_bottom` is the world origin of cell (0,0).
pub(crate) fn rasterize_polygons(
    grid_w: usize,
    grid_h: usize,
    pixel_size: f64,
    left_bottom: Point,
    polygons: &[Polygon],
) -> Vec<u8> {
    let mut data = vec![0u8; grid_w * grid_h];
    if pixel_size <= 0.0 {
        return data;
    }
    let mut outline = Outline::new();
    let to_sub = |p: Point| -> (i32, i32) {
        (
            upscale((p.x() - left_bottom.x()) as f64 / pixel_size),
            upscale((p.y() - left_bottom.y()) as f64 / pixel_size),
        )
    };
    for polygon in polygons {
        let pts: Vec<Point> = polygon.points().to_vec();
        if pts.len() < 2 {
            continue;
        }
        let (mut px, mut py) = to_sub(pts[0]);
        let (fx, fy) = (px, py);
        for pt in pts.iter().skip(1) {
            let (nx, ny) = to_sub(*pt);
            outline.line(px, py, nx, ny);
            px = nx;
            py = ny;
        }
        outline.line(px, py, fx, fy);
    }

    // `sweep_scanline`: group cells by (y, x), accumulate cover/area, emit
    // cells/spans where alpha > 0.
    let mut by_row: std::collections::BTreeMap<i32, Vec<(i32, i64, i64)>> =
        std::collections::BTreeMap::new();
    for (&(x, y), &c) in &outline.cells {
        if c.area != 0 || c.cover != 0 {
            by_row.entry(y).or_default().push((x, c.area, c.cover));
        }
    }
    for (y, mut cells) in by_row {
        if y < 0 || y as usize >= grid_h {
            continue;
        }
        cells.sort_by_key(|&(x, _, _)| x);
        let mut cover: i64 = 0;
        let mut i = 0;
        while i < cells.len() {
            let x = cells[i].0;
            let mut area = cells[i].1;
            cover += cells[i].2;
            i += 1;
            // Accumulate all cells with the same X.
            while i < cells.len() && cells[i].0 == x {
                area += cells[i].1;
                cover += cells[i].2;
                i += 1;
            }
            // `x++` happens only inside `if(area)` (`agg_rasterizer_scanline_aa.h:241`).
            let mut next_x = x;
            if area != 0 {
                let alpha = calculate_alpha((cover << (SUBPIXEL_SHIFT + 1)) - area);
                if alpha > 0 && x >= 0 && (x as usize) < grid_w {
                    data[y as usize * grid_w + x as usize] = 1;
                }
                next_x = x + 1;
            }
            if i < cells.len() && cells[i].0 > next_x {
                let alpha = calculate_alpha(cover << (SUBPIXEL_SHIFT + 1));
                if alpha > 0 {
                    for cx in next_x..cells[i].0 {
                        if cx >= 0 && (cx as usize) < grid_w {
                            data[y as usize * grid_w + cx as usize] = 1;
                        }
                    }
                }
            }
        }
    }
    data
}

/// `dilate_trimming_region` (`SupportMaterial.cpp:980-999`): 8-neighborhood
/// AND across the 3 rows above/at/below — a cell stays set only if all 9
/// neighbours in `trimming` are set. Applied to the trimming mask before
/// `seed_fill_block`.
pub(crate) fn dilate_trimming_region(trimming: &[u8], grid_w: usize, grid_h: usize) -> Vec<u8> {
    let mut dilated = vec![0u8; trimming.len()];
    for r in 1..grid_h - 1 {
        for c in 1..grid_w - 1 {
            let mut addr = c + (r - 1) * grid_w;
            let mut b = trimming[addr - 1] != 0 && trimming[addr] != 0 && trimming[addr + 1] != 0;
            addr += grid_w;
            b = b && trimming[addr - 1] != 0 && trimming[addr] != 0 && trimming[addr + 1] != 0;
            addr += grid_w;
            b = b && trimming[addr - 1] != 0 && trimming[addr] != 0 && trimming[addr + 1] != 0;
            dilated[addr - grid_w] = b as u8;
        }
    }
    dilated
}

/// `seed_fill_block` (`SupportMaterial.cpp:1001-1040`): propagate the
/// rasterized support pixels over each `(oversampling x oversampling)` macro
/// block up to the dilated trimming mask.
#[allow(clippy::too_many_arguments)]
pub(crate) fn seed_fill_block(
    grid: &mut [u8],
    grid_w: usize,
    trimming: &[u8],
    grid_blocks_x: usize,
    grid_blocks_y: usize,
    oversampling: usize,
) {
    let size = oversampling;
    let stride = grid_w;
    for block_r in 0..grid_blocks_y {
        for block_c in 0..grid_blocks_x {
            let base = block_c * size + 1 + (block_r * size + 1) * stride;
            // Top to bottom propagation.
            for r in 0..size {
                if r > 0 {
                    for c in 0..size {
                        propagate(grid, trimming, base, r, c, stride, -(stride as isize));
                    }
                }
                for c in 1..size {
                    propagate(grid, trimming, base, r, c, stride, -1);
                }
                for c in (0..size - 1).rev() {
                    propagate(grid, trimming, base, r, c, stride, 1);
                }
            }
            // Bottom to top propagation.
            for r in (0..size - 1).rev() {
                for c in 0..size {
                    propagate(grid, trimming, base, r, c, stride, stride as isize);
                }
                for c in 1..size {
                    propagate(grid, trimming, base, r, c, stride, -1);
                }
                for c in (0..size - 1).rev() {
                    propagate(grid, trimming, base, r, c, stride, 1);
                }
            }
        }
    }
}

/// `PROPAGATION_STEP` (`:1017-1023`): copy a filled neighbour cell into an
/// un-filled, un-trimmed cell.
fn propagate(
    grid: &mut [u8],
    trimming: &[u8],
    base: usize,
    r: usize,
    c: usize,
    stride: usize,
    offset: isize,
) {
    let addr = base + r * stride + c;
    let addr2 = (addr as isize + offset) as usize;
    if grid[addr2] != 0 && trimming[addr] == 0 && trimming[addr2] == 0 {
        grid[addr] = 1;
    }
}

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
            for k in lo..hi {
                let idx = start_to_idx[k].1;
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
