//! `emit_loops_in_band` — the contour-band arch emitter used by
//! `Fill::connect_base_support` (sparse support infill connection),
//! ported from `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:1952-2140`.
//!
//! Traces a closed contour from `tbegin` to `tend` (contour
//! parameter fractions) and emits the arches that stay inside the
//! vertical band `[left, right]`, inserting vertical connector
//! segments where the contour crosses the band edges.

use crate::geometry::{Coord, Point, Polyline};

#[expect(dead_code, reason = "wired by the raft fills slice")]
mod arches;
#[expect(dead_code, reason = "wired by the raft fills slice")]
pub(crate) use connect::connect_base_support;
mod caps;
mod connect;
mod extend;
mod link;
mod mark;
mod probes;
mod take;
mod tests;

/// Port of `emit_loops_in_band` (`FillBase.cpp:1952`).
///
/// `contour` is a closed polygon; `contour_params[i]` is the
/// accumulated length at vertex `i` (with `contour_params[len]` the
/// perimeter); `tbegin`/`tend` are parameter values; arches shorter
/// than `min_length` are dropped.
#[allow(clippy::too_many_arguments)]
pub(crate) fn emit_loops_in_band(
    left: Coord,
    right: Coord,
    contour: &[Point],
    contour_params: &[f64],
    tbegin: f64,
    tend: f64,
    min_length: f64,
    polylines_out: &mut Vec<Polyline>,
) {
    debug_assert!(left < right);
    debug_assert_eq!(contour.len() + 1, contour_params.len());
    debug_assert!(contour.len() >= 3);

    // Find the segments containing tbegin / tend (`:1984-1997`):
    // both lower_bound; only the BEGIN iterator walks one back when
    // the parameter does not land exactly on a vertex.
    let mut ibegin = segment_begin_of(contour_params, tbegin);
    let mut iend = partition_point(contour_params, tend);
    if ibegin == contour.len() {
        ibegin = 0;
    }
    if iend == contour.len() {
        iend = 0;
    }
    debug_assert_ne!(ibegin, iend);

    // Trim the start and end segments to the parameter positions.
    let pbegin = interpolate(contour, contour_params, ibegin, tbegin, true);
    let pend = interpolate(contour, contour_params, iend, tend, false);

    let mut state = BandState::new(left, right, min_length, polylines_out);
    let mut p1 = pbegin;
    state.side1 = side_of(p1.x(), left, right);
    if state.side1 == Side::Mid {
        state.add_inner_point(p1);
    }

    let mut index = ibegin;
    while index != iend {
        let mut inext = index + 1;
        if inext == contour.len() {
            inext = 0;
        }
        let p2 = if inext == iend { pend } else { contour[inext] };
        state.side2 = side_of(p2.x(), left, right);
        match (state.side1, state.side2) {
            (Side::Mid, Side::Mid) => state.add_inner_point(p2),
            (Side::Mid, _) => {
                // Inside → outside.
                state.add_interpolated_point(p1, p2, state.side2, InOut::Leaving);
                state.add_outer_point(p2);
            }
            (_, Side::Mid) => {
                // Outside → inside.
                state.add_interpolated_point(p1, p2, state.side1, InOut::Entering);
                state.add_inner_point(p2);
            }
            (first, second) if first != second => {
                // Both outside, crossing the band.
                state.add_interpolated_point(p1, p2, first, InOut::Entering);
                state.add_interpolated_point(p1, p2, second, InOut::Leaving);
            }
            _ => state.add_outer_point(p2),
        }
        state.side1 = state.side2;
        p1 = p2;
        index = inext;
    }
    state.finalize();
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Side {
    Left,
    Right,
    Mid,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum InOut {
    Entering,
    Leaving,
}

fn side_of(x: Coord, left: Coord, right: Coord) -> Side {
    if x < left {
        Side::Left
    } else if x > right {
        Side::Right
    } else {
        Side::Mid
    }
}

fn partition_point(params: &[f64], t: f64) -> usize {
    let mut low = 0usize;
    let mut high = params.len();
    while low < high {
        let mid = (low + high) / 2;
        if params[mid] < t {
            low = mid + 1;
        } else {
            high = mid;
        }
    }
    low
}

/// The begin segment index: the lower-bound position, walked one
/// back unless the parameter lands exactly on a vertex.
fn segment_begin_of(params: &[f64], t: f64) -> usize {
    let index = partition_point(params, t);
    if params[index] != t {
        index.saturating_sub(1)
    } else {
        index
    }
}

/// Interpolate the point at parameter `t` on the segment containing
/// it; `forward` walks toward `ibegin + 1`, otherwise toward
/// `iend - 1` (upstream `:1994-2001`).
fn interpolate(contour: &[Point], params: &[f64], index: usize, t: f64, forward: bool) -> Point {
    let (other, t1, t2) = if forward {
        let next = (index + 1) % contour.len();
        (next, params[index], params[index + 1])
    } else {
        let prev = if index == 0 {
            contour.len() - 1
        } else {
            index - 1
        };
        (prev, params[index], params[prev])
    };
    let ratio = ((t - t1) / (t2 - t1)).clamp(0.0, 1.0);
    let a = contour[index];
    let b = contour[other];
    Point::new(
        a.x() + ((b.x() - a.x()) as f64 * ratio).round() as Coord,
        a.y() + ((b.y() - a.y()) as f64 * ratio).round() as Coord,
    )
}

struct BandState<'a> {
    left: Coord,
    right: Coord,
    min_length: f64,
    polylines_out: &'a mut Vec<Polyline>,
    polyline: Vec<Point>,
    polyline_end: usize,
    side1: Side,
    side2: Side,
}

impl<'a> BandState<'a> {
    fn new(
        left: Coord,
        right: Coord,
        min_length: f64,
        polylines_out: &'a mut Vec<Polyline>,
    ) -> Self {
        Self {
            left,
            right,
            min_length,
            polylines_out,
            polyline: Vec::new(),
            polyline_end: 0,
            side1: Side::Mid,
            side2: Side::Mid,
        }
    }

    fn add_inner_point(&mut self, point: Point) {
        self.polyline.push(point);
    }

    fn add_outer_point(&mut self, point: Point) {
        if self.polyline_end > 0 {
            self.polyline.push(point);
        }
    }

    fn add_interpolated_point(&mut self, p1: Point, p2: Point, side: Side, inout: InOut) {
        debug_assert!(side == Side::Left || side == Side::Right);
        let x = if side == Side::Left {
            self.left
        } else {
            self.right
        };
        let y = p1.y()
            + ((x - p1.x()) as f64 * (p2.y() - p1.y()) as f64 / (p2.x() - p1.x()) as f64).round()
                as Coord;
        let point = Point::new(x, y);
        match inout {
            InOut::Leaving => {
                debug_assert_eq!(self.polyline_end, 0);
                self.polyline_end = self.polyline.len();
                self.polyline.push(point);
            }
            InOut::Entering => {
                if self.polyline_end > 0 {
                    if (self.side1 == Side::Left) == (y - self.polyline[self.polyline_end].y() < 0)
                    {
                        // Emit the vertical segment: remove the split point.
                        self.polyline.remove(self.polyline_end);
                    } else {
                        // Don't emit the vertical segment; split the contour.
                        self.finalize();
                        self.polyline.push(point);
                    }
                    self.polyline_end = 0;
                } else {
                    self.polyline.push(point);
                }
            }
        }
    }

    fn finalize(&mut self) {
        if self.polyline_end > 0 {
            self.polyline.truncate(self.polyline_end);
        }
        if self.polyline.len() > 1 {
            let merge_with_last = self.polylines_out.last().is_some_and(|last| {
                let a = last.points().last().expect("non-empty polyline");
                let b = &self.polyline[0];
                let (dx, dy) = (a.x() - b.x(), a.y() - b.y());
                dx * dx + dy * dy < 16
            });
            if merge_with_last {
                let last = self.polylines_out.last_mut().expect("checked above");
                last.append_points(self.polyline.iter().skip(1).copied());
            } else if polyline_length(&self.polyline) > self.min_length {
                self.polylines_out
                    .push(Polyline::new(std::mem::take(&mut self.polyline)));
                return;
            }
        }
        self.polyline.clear();
    }
}

fn polyline_length(points: &[Point]) -> f64 {
    points
        .windows(2)
        .map(|pair| {
            let (dx, dy) = (pair[1].x() - pair[0].x(), pair[1].y() - pair[0].y());
            ((dx * dx + dy * dy) as f64).sqrt()
        })
        .sum()
}
