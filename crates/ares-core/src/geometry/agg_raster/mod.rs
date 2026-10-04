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

mod cells;
mod contours;
mod grid_ops;

use crate::geometry::{Point, Polygon};
use cells::Outline;
pub(crate) use contours::contours_simplified;
pub(crate) use grid_ops::{dilate_trimming_region, seed_fill_block};

pub(crate) struct RasterGeometry {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) pixel_size: f64,
    pub(crate) left_bottom: Point,
}

pub(super) const SUBPIXEL_SHIFT: i32 = 8;
pub(super) const SUBPIXEL_SCALE: i32 = 1 << SUBPIXEL_SHIFT;
pub(super) const SUBPIXEL_MASK: i32 = SUBPIXEL_SCALE - 1;
pub(super) const AA_SHIFT: i32 = 8;
pub(super) const AA_MASK: i32 = (1 << AA_SHIFT) - 1;
pub(super) const DX_LIMIT: i64 = 16384 << SUBPIXEL_SHIFT;

/// `iround` (`agg_basics.h:131`): round-half-away to the subpixel grid.
/// `ras_conv_int::upscale` (`agg_rasterizer_sl_clip.h:38`) =
/// `iround(v * poly_subpixel_scale)`, where `iround` (`agg_basics.h:188`) is
/// `int(v < 0 ? v - 0.5 : v + 0.5)` — round-half-AWAY-from-zero, not
/// `+0.5 floor` (which truncates negative halves toward zero).
pub(super) fn upscale(v: f64) -> i32 {
    let scaled = v * SUBPIXEL_SCALE as f64;
    (if scaled < 0.0 {
        scaled - 0.5
    } else {
        scaled + 0.5
    }) as i32
}
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
    for (y, cells) in by_row {
        if y < 0 || y as usize >= grid_h {
            continue;
        }
        sweep_scanline(
            &mut data[y as usize * grid_w..(y as usize + 1) * grid_w],
            cells,
        );
    }
    data
}

fn sweep_scanline(row: &mut [u8], mut cells: Vec<(i32, i64, i64)>) {
    let grid_w = row.len();
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
                row[x as usize] = 1;
            }
            next_x = x + 1;
        }
        if i >= cells.len() || cells[i].0 <= next_x {
            continue;
        }
        let alpha = calculate_alpha(cover << (SUBPIXEL_SHIFT + 1));
        if alpha <= 0 {
            continue;
        }
        for cx in next_x..cells[i].0 {
            if cx >= 0 && (cx as usize) < grid_w {
                row[cx as usize] = 1;
            }
        }
    }
}
