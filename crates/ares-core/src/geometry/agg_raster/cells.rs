//! AGG `rasterizer_cells_aa` cell accumulation (`agg_rasterizer_cells_aa.h`).

use super::{DX_LIMIT, SUBPIXEL_MASK, SUBPIXEL_SCALE, SUBPIXEL_SHIFT};

#[derive(Clone, Copy, Default)]
pub(super) struct Cell {
    pub(super) cover: i64,
    pub(super) area: i64,
}

/// Accumulate `area`/`cover` for polygon edges (a `rasterizer_cells_aa`
/// outline). Cells are keyed by `(x, y)`; insertion order is irrelevant
/// because `sweep_scanline` re-sorts per scanline.
pub(super) struct Outline {
    pub(super) cells: std::collections::HashMap<(i32, i32), Cell>,
    // Running cell the AGG accumulator writes into (`m_curr_cell`).
    curr: (i32, i32),
    min_y: i32,
    max_y: i32,
}

impl Outline {
    pub(super) fn new() -> Self {
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
    pub(super) fn line(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) {
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
