//! `marchsq` ring-scanning + edge interpolation (`MarchingSquares.hpp`),
//! split from `marching_squares.rs` for the <400 LOC rule.

use super::{Coord, Dir, Grid, step};

impl<F: Fn(i64, i64) -> f32> Grid<F> {
    fn scan_rings(&mut self) -> Vec<Vec<Coord>> {
        let mut rings = Vec::new();
        let mut startidx = 0usize;
        // Bound the outer ring scan to `gridlen` iterations — a well-formed
        // grid consumes dirs monotonically; the bound guards against a
        // degenerate ring leaving `search_start_cell` stuck on one cell.
        for _ in 0..self.gridlen + 1 {
            let found = self.search_start_cell(startidx);
            if found >= self.gridlen || found < startidx {
                break;
            }
            startidx = found;
            let mut ring = Vec::new();
            let mut idx = startidx;
            let mut next = self.next_dir(idx, Dir::All);
            // Guard: a well-formed grid always closes the ring back at
            // startidx; bail on a degenerate/stuck direction to avoid a hang.
            let max_steps = self.gridlen * 4 + 64;
            for _ in 0..max_steps {
                if next == Dir::None || next == Dir::All {
                    break;
                }
                ring.push(Coord {
                    r: idx as i64,
                    c: next as i64,
                });
                self.clr_dirs(idx, next);
                idx = self.stepidx(idx, next);
                next = self.next_dir(idx, next);
                if idx == startidx {
                    break;
                }
            }
            // Clear residual direction bits on the current cell AND the start
            // cell so the outer search advances past a degenerate ring instead
            // of revisiting it (upstream relies on rings always closing).
            self.clr_dirs(idx, Dir::from_bits(0b1111));
            self.clr_dirs(startidx, Dir::from_bits(0b1111));
            if ring.len() > 1 {
                rings.push(ring);
            }
        }
        rings
    }

    // `tl`/`bl`/`br`/`tr` — the cell's four raster-coordinate corners.
    fn tl(&self, crd: Coord) -> Coord {
        self.rastercoord(crd)
    }
    fn bl(&self, crd: Coord) -> Coord {
        let t = self.tl(crd);
        Coord {
            r: t.r + self.window.r,
            c: t.c,
        }
    }
    fn br(&self, crd: Coord) -> Coord {
        let t = self.tl(crd);
        Coord {
            r: t.r + self.window.r,
            c: t.c + self.window.c,
        }
    }
    fn tr(&self, crd: Coord) -> Coord {
        let t = self.tl(crd);
        Coord {
            r: t.r,
            c: t.c + self.window.c,
        }
    }

    // `edge(ringvertex)` — the (from, to) raster endpoints of the crossed cell
    // edge; `from.dir` is the direction the interpolation walks along it.
    fn edge(&self, ringvertex: Coord) -> (Coord, Coord, Dir) {
        let cell = self.coord(ringvertex.r as usize);
        let d = Dir::from_bits(ringvertex.c as u32);
        let (from, to, dir) = match d {
            Dir::Left => (self.tl(cell), self.bl(cell), Dir::Down),
            Dir::Down => (self.bl(cell), self.br(cell), Dir::Right),
            Dir::Right => (self.br(cell), self.tr(cell), Dir::Up),
            Dir::Up => (self.tr(cell), self.tl(cell), Dir::Left),
            _ => unreachable!("ring vertex carries a cell-exit direction"),
        };
        let mut to = to;
        step(&mut to, dir, 1);
        (
            Coord {
                r: from.r.clamp(0, self.rastsize.r - 1),
                c: from.c.clamp(0, self.rastsize.c - 1),
            },
            Coord {
                r: to.r.clamp(0, self.rastsize.r),
                c: to.c.clamp(0, self.rastsize.c),
            },
            dir,
        )
    }

    // `lower_bound`-style binary search along the edge for the first raster
    // point whose value >= isoval, then shift bottom/right out by one to
    // account for raster pixel width (`interpolate_edge`, MarchingSquares.hpp).
    fn interpolate_edge(&self, ecrd: &mut Coord, isoval: f32) {
        let (from, to, dir) = self.edge(*ecrd);
        // Distance (in raster-pixel steps) from `from` to `to` along `dir`.
        let len = (Coord {
            r: to.r - from.r,
            c: to.c - from.c,
        })
        .manhattan_along(dir);
        let mut lo = 0i64;
        let mut hi = len; // count of steps before the first `>= isoval`
        let mut first = from;
        // `lower_bound(first, to, isoval)`: partition [0,len) by `isoval < v`.
        while lo < hi {
            let mid = lo + (hi - lo) / 2;
            let mut probe = from;
            step(&mut probe, dir, mid);
            let v = if self.is_within(probe) {
                self.isoval(probe)
            } else {
                f32::MIN
            };
            if v < isoval {
                lo = mid + 1;
            } else {
                hi = mid;
            }
        }
        step(&mut first, dir, lo);
        let mut ec = first;
        if dir == Dir::Up {
            ec.r += 1;
        } else if dir == Dir::Left {
            ec.c += 1;
        }
        *ecrd = ec;
    }

    fn interpolate_rings(&self, rings: &mut [Vec<Coord>], isoval: f32) {
        for ring in rings {
            for e in ring {
                self.interpolate_edge(e, isoval);
            }
        }
    }
}

/// `marchsq::execute` — returns rings of `{r: cell idx, c: Dir}` vertices.
/// `field(row, col)` gives the raster value; `window` is the cell size in
/// raster pixels.
pub(crate) fn execute<F: Fn(i64, i64) -> f32>(
    field: F,
    rows: i64,
    cols: i64,
    isoval: f32,
    window: Coord,
) -> Vec<Vec<Coord>> {
    if rows == 0 || cols == 0 {
        return Vec::new();
    }
    let mut grid = Grid::new(field, rows, cols, window);
    grid.tag_grid(isoval);
    let mut rings = grid.scan_rings();
    grid.interpolate_rings(&mut rings, isoval);
    rings
}
