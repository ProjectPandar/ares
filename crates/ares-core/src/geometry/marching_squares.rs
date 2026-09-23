//! `marchsq` marching-squares (`src/libslic3r/MarchingSquares.hpp`): bit-packed
//! contour extraction over a scalar-field raster, used by `FillTpmsFK`. The
//! grid tags each raster cell's top-left `a` corner into `u32` words; the
//! `NEXT_CCW` table maps a cell's `<cbda>` corner bitmap to the CCW ring-exit
//! directions `<urdl>`; `scan_rings` marches rings clearing direction bits;
//! `interpolate_rings` binary-searches each ring edge for the isovalue.
//!
//! Ported verbatim (the bit packing and the ambiguous-cell `next_dir` rule are
//! observable in the emitted point order). The raster `Field` is injected as a
//! closure `fn(row, col) -> f32` returning the field value.

use crate::geometry::Point;

/// `marchsq::Coord` — a raster/grid coordinate (`r` row, `c` col).
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Coord {
    pub(crate) r: i64,
    pub(crate) c: i64,
}

impl Coord {
    fn seq(self, res: Coord) -> usize {
        (self.r * res.c + self.c) as usize
    }

    // Raster-pixel distance from `self` to `other` along the single axis that
    // `dir` walks (Left/Right change `c`, Up/Down change `r`).
    fn manhattan_along(self, dir: Dir) -> i64 {
        let d = match dir {
            Dir::Left | Dir::Right => self.c,
            Dir::Up | Dir::Down => self.r,
            _ => 0,
        };
        d.abs()
    }
}

/// `marchsq::Dir` — the `<urdl>` cell-exit direction bits.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u8)]
enum Dir {
    None = 0b0000,
    Left = 0b0001,
    Down = 0b0010,
    Right = 0b0100,
    LeftRight = 0b0101,
    Up = 0b1000,
    UpDown = 0b1010,
    All = 0b1111,
}

impl Dir {
    fn from_bits(v: u32) -> Dir {
        match v & 0b1111 {
            0b0001 => Dir::Left,
            0b0010 => Dir::Down,
            0b0100 => Dir::Right,
            0b0101 => Dir::LeftRight,
            0b1000 => Dir::Up,
            0b1010 => Dir::UpDown,
            0b0000 => Dir::None,
            _ => Dir::All,
        }
    }
    fn bits(self) -> u32 {
        self as u32
    }
}

/// `NEXT_CCW` — `<cbda>` tag bitmap → `<urdl>` exit directions.
const NEXT_CCW: [u32; 16] = {
    let mut map = [0u32; 16];
    let mut cbda = 0u32;
    while cbda < 16 {
        let dcba = (cbda & 0b0001) | ((cbda >> 1) & 0b0110) | ((cbda << 2) & 0b1000);
        let adcb = (dcba >> 1) | ((dcba << 3) & 0b1000);
        map[cbda as usize] = !dcba & adcb;
        cbda += 1;
    }
    map
};

pub(super) fn step(crd: &mut Coord, d: Dir, n: i64) {
    match d {
        Dir::Left => crd.c -= n,
        Dir::Down => crd.r += n,
        Dir::Right => crd.c += n,
        Dir::Up => crd.r -= n,
        _ => {}
    }
}

/// `marchsq::Grid` over a field `F: Fn(i64 row, i64 col) -> f32` with `rows`/`cols`.
pub(crate) struct Grid<F: Fn(i64, i64) -> f32> {
    field: F,
    rows: i64,
    cols: i64,
    window: Coord,
    rastsize: Coord,
    gridsize: Coord,
    gridlen: usize,
    tags: Vec<u32>,
    dirs: Vec<u32>,
}

impl<F: Fn(i64, i64) -> f32> Grid<F> {
    pub(super) fn new(field: F, rows: i64, cols: i64, window: Coord) -> Self {
        let rastsize = Coord { r: rows, c: cols };
        let gridsize = Coord {
            r: 2 + rastsize.r / window.r,
            c: 2 + rastsize.c / window.c,
        };
        let gridlen = (gridsize.r * gridsize.c) as usize;
        Self {
            field,
            rows,
            cols,
            window,
            rastsize,
            gridsize,
            gridlen,
            tags: vec![0; gridlen / 32 + 1],
            dirs: vec![0; gridlen / 8 + 1],
        }
    }

    fn rastercoord(&self, crd: Coord) -> Coord {
        Coord {
            r: (crd.r - 1) * self.window.r,
            c: (crd.c - 1) * self.window.c,
        }
    }

    fn coord(&self, i: usize) -> Coord {
        Coord {
            r: i as i64 / self.gridsize.c,
            c: i as i64 % self.gridsize.c,
        }
    }

    fn seq(&self, crd: Coord) -> usize {
        crd.seq(self.gridsize)
    }

    fn is_within(&self, crd: Coord) -> bool {
        crd.r >= 0 && crd.r < self.rastsize.r && crd.c >= 0 && crd.c < self.rastsize.c
    }

    fn isoval(&self, crd: Coord) -> f32 {
        (self.field)(crd.r, crd.c)
    }

    fn get_tags_block32(&self, bidx: usize, v: f32) -> u32 {
        let mut gcrd = self.coord(bidx * 32);
        let mut rcrd = self.rastercoord(gcrd);
        let mut tags = 0u32;
        let mut b = 1u32;
        while b != 0 {
            if self.is_within(rcrd) && self.isoval(rcrd) > v {
                tags |= b;
            }
            gcrd.c += 1;
            rcrd.c += self.window.c;
            if gcrd.c >= self.gridsize.c {
                gcrd = Coord {
                    r: gcrd.r + 1,
                    c: 0,
                };
                rcrd = self.rastercoord(gcrd);
            }
            b <<= 1;
        }
        tags
    }

    fn get_tags9(&self, gidx: usize) -> u32 {
        let mut tags = 0u32;
        let i = gidx / 32;
        let o = gidx % 32;
        if gidx < self.gridlen {
            tags = (self.tags[i] >> o) & 0x1ff;
            if o > 32 - 9 && i + 1 < self.tags.len() {
                tags |= (self.tags[i + 1] << (32 - o)) & 0x1ff;
            }
        }
        tags
    }

    fn get_dirs_block8(&self, bidx: usize) -> u32 {
        let gidx = bidx * 8;
        let mut dirs = 0u32;
        let tags9 = self.get_tags9(gidx) | (self.get_tags9(gidx + self.gridsize.c as usize) << 16);
        if tags9 != 0 && tags9 != 0x01ff01ff {
            let mut t = tags9;
            let mut s = 0;
            while s < 32 {
                let tags = (t & 0b11) | ((t >> 14) & 0b1100);
                dirs |= NEXT_CCW[tags as usize] << s;
                t >>= 1;
                s += 4;
            }
        }
        dirs
    }

    fn clr_dirs(&mut self, gidx: usize, d: Dir) {
        let i = gidx / 8;
        let o = gidx % 8;
        self.dirs[i] &= !(d.bits() << (o * 4));
    }

    fn get_dirs(&self, gidx: usize) -> Dir {
        let i = gidx / 8;
        let o = gidx % 8;
        Dir::from_bits((self.dirs[i] >> (o * 4)) & 0b1111)
    }

    fn stepidx(&self, idx: usize, d: Dir) -> usize {
        match d {
            Dir::Left => idx - 1,
            Dir::Down => idx + self.gridsize.c as usize,
            Dir::Right => idx + 1,
            Dir::Up => idx - self.gridsize.c as usize,
            _ => idx,
        }
    }

    fn search_start_cell(&self, gidx: usize) -> usize {
        let mut i = gidx / 8;
        let mut o = gidx % 8;
        while i < self.dirs.len() {
            if self.dirs[i] == 0 {
                i += 1;
                o = 0;
            } else {
                let mut m = 0b1111u32 << (o * 4);
                while self.dirs[i] & m == 0 {
                    m <<= 4;
                    o += 1;
                }
                break;
            }
        }
        i * 8 + o
    }

    fn next_dir(&self, idx: usize, prev: Dir) -> Dir {
        let next = self.get_dirs(idx);
        match next {
            Dir::LeftRight => {
                if prev == Dir::Up {
                    Dir::Left
                } else {
                    Dir::Right
                }
            }
            Dir::UpDown => {
                if prev == Dir::Right {
                    Dir::Up
                } else {
                    Dir::Down
                }
            }
            _ => next,
        }
    }

    pub(super) fn tag_grid(&mut self, isoval: f32) {
        for bidx in 0..self.tags.len() {
            self.tags[bidx] = self.get_tags_block32(bidx, isoval);
        }
        for bidx in 0..self.dirs.len() {
            self.dirs[bidx] = self.get_dirs_block8(bidx);
        }
    }
}

mod rings;
pub(crate) use rings::execute;
