//! Post-extension linking phases of `connect_base_support` — ported
//! from `OrcaSlicer/src/libslic3r/Fill/FillBase.cpp:2425-2502` and
//! `:2577-2607`: vertical-arch consumption `:2425-2455`,
//! cost-classified arch selection `:2463-2502`, zig-zag traversal
//! `:2577-2592` and left caps `:2596-2607` (the very-long-arch
//! emission lives in [`super::caps`], the T-joints follow here).

use super::arches::{SCALED_EPSILON, SupportArcCost, evaluate_support_arches};
use super::take::{resolve_merged, take_next};
use crate::fill::connect::types::{BoundaryContour, Intersection};
use crate::geometry::Point;

/// Shared mutable state of the linking phases.
pub(super) struct LinkPhases<'a> {
    boundary: &'a [BoundaryContour],
    intersections: &'a mut [Intersection],
    paths: &'a mut [Option<Vec<Point>>],
    merged_with: &'a mut [usize],
    line_half_width: f64,
    line_spacing: f64,
    min_arch_length: f64,
    trim_length: f64,
}

impl<'a> LinkPhases<'a> {
    /// The source phase block carries the same scalars.
    #[expect(
        clippy::too_many_arguments,
        reason = "the source phase block carries the same scalars"
    )]
    pub(super) fn new(
        boundary: &'a [BoundaryContour],
        intersections: &'a mut [Intersection],
        paths: &'a mut [Option<Vec<Point>>],
        merged_with: &'a mut [usize],
        line_half_width: f64,
        line_spacing: f64,
        min_arch_length: f64,
        trim_length: f64,
    ) -> LinkPhases<'a> {
        LinkPhases {
            boundary,
            intersections,
            paths,
            merged_with,
            line_half_width,
            line_spacing,
            min_arch_length,
            trim_length,
        }
    }

    fn take(&mut self, index: usize, take_first: bool) {
        take_next(
            self.boundary,
            self.intersections,
            self.paths,
            self.merged_with,
            index,
            take_first,
            self.line_half_width,
            self.trim_length,
        );
    }

    fn could_take_prev(&self, index: usize) -> bool {
        !self.intersections[index].consumed
            && self.intersections[index].not_taken_prev > SCALED_EPSILON
    }

    fn could_take_next(&self, index: usize) -> bool {
        !self.intersections[index].consumed
            && self.intersections[index].not_taken_next > SCALED_EPSILON
    }

    fn same_chain(&self, left: usize, right: usize) -> bool {
        resolve_merged(self.merged_with, left) == resolve_merged(self.merged_with, right)
    }

    /// Vertical-arch consumption (`:2425-2455`).
    pub(super) fn consume_vertical_arches(&mut self) {
        for index in 0..self.intersections.len() {
            self.trace_order_debug(index);
            if self.intersections[index].consumed {
                continue;
            }
            let other = index ^ 1;
            let (prev, next) = (
                self.intersections[index].prev,
                self.intersections[index].next,
            );
            let can_take_prev = prev.is_some_and(|prev| {
                vertical_dir(self.boundary, self.intersections, index, prev)
                    && !self.intersections[prev].consumed
                    && prev != other
            });
            let can_take_next = next.is_some_and(|next| {
                vertical_dir(self.boundary, self.intersections, index, next)
                    && !self.intersections[next].consumed
                    && next != other
            });
            if can_take_prev && (!can_take_next || take_vertical_prev(self.intersections, index)) {
                let prev = self.intersections[index].prev.expect("checked above");
                self.take_vertical(prev, false);
            } else if can_take_next {
                self.take_vertical(index, true);
            }
        }
    }

    #[cfg(test)]
    fn trace_order_debug(&self, index: usize) {
        if std::env::var("ARES_ORDER_DEBUG").is_ok() && index < 8 {
            let i = &self.intersections[index];
            eprintln!(
                "ORDR idx={index} consumed={} prev_trim={} prev_len={:.3} next_trim={} next_len={:.3}",
                i.consumed, i.prev_trimmed, i.not_taken_prev, i.next_trimmed, i.not_taken_next
            );
        }
    }

    #[cfg(not(test))]
    fn trace_order_debug(&self, _index: usize) {}

    /// Take a vertical arch unless it is trimmed below
    /// `min_arch_length` (`:2448-2455`).
    fn take_vertical(&mut self, index: usize, take_first: bool) {
        let (trimmed, not_taken) = if take_first {
            (
                self.intersections[index].next_trimmed,
                self.intersections[index].not_taken_next,
            )
        } else {
            (
                self.intersections[index].prev_trimmed,
                self.intersections[index].not_taken_prev,
            )
        };
        if !trimmed || not_taken > self.min_arch_length {
            self.take(index, take_first);
        }
    }

    /// Cost-classified arch selection (`evaluate_support_arches`
    /// `:2203-2243` + selection `:2463-2502`). Returns the arc costs
    /// for the later T-joint and very-long-arch phases (upstream
    /// reuses the same table without recomputation).
    pub(super) fn select_support_arches(&mut self) -> Vec<SupportArcCost> {
        let arches = evaluate_support_arches(
            self.boundary,
            self.intersections,
            self.paths,
            SCALED_EPSILON,
        );
        let cost_low = self.line_spacing * 1.3;
        let cost_high = self.line_spacing * 2.0;
        let mut selected = Vec::new();
        for index in 0..self.intersections.len() {
            if self.intersections[index].consumed {
                continue;
            }
            let (cost_prev, cost_next) = (arches[index * 2].cost, arches[index * 2 + 1].cost);
            let (cost_min, cost_max) = if cost_prev < cost_next {
                (cost_prev, cost_next)
            } else {
                (cost_next, cost_prev)
            };
            if cost_max < cost_low || cost_min > cost_high {
                continue;
            }
            if (cost_max - cost_min) / cost_max < 0.25 {
                continue;
            }
            if cost_prev > cost_low {
                selected.push((cost_prev, index, true));
            }
            if cost_next > cost_low {
                selected.push((cost_next, index, false));
            }
        }
        // Take the longest arch first.
        selected.sort_by(|left, right| right.0.partial_cmp(&left.0).expect("finite costs"));
        for (_, index, prev) in selected {
            if self.intersections[index].consumed {
                continue;
            }
            self.take_selected(index, prev);
        }
        arches
    }

    fn take_selected(&mut self, index: usize, prev: bool) {
        if prev {
            let prev_index = self.intersections[index]
                .prev
                .expect("selected arcs have links");
            self.take(prev_index, false);
        } else {
            self.take(index, true);
        }
    }

    /// Zig-zag traversal (`:2577-2592`): traverse the unconnected
    /// lines left to right, linking each line to its neighbour
    /// unless both already sit on the same merged chain.
    pub(super) fn link_zigzag(&mut self) {
        for index in 0..self.intersections.len() {
            if self.intersections[index].consumed {
                continue;
            }
            if index % 2 == 0 {
                self.link_zigzag_forward(index);
            } else {
                self.link_zigzag_backward(index);
            }
        }
    }

    fn link_zigzag_forward(&mut self, index: usize) {
        let Some(next) = self.intersections[index].next else {
            return;
        };
        if !self.same_chain(index, next) {
            self.take(index, true);
        }
    }

    fn link_zigzag_backward(&mut self, index: usize) {
        let Some(prev) = self.intersections[index].prev else {
            return;
        };
        if !self.same_chain(index, prev) {
            self.take(prev, false);
        }
    }

    /// Left caps (`:2596-2607`): close self loops at both ends of
    /// each infill line.
    pub(super) fn close_left_caps(&mut self) {
        for index in 0..self.intersections.len() {
            self.close_left_cap(index, true);
            self.close_left_cap(index, false);
        }
    }

    fn close_left_cap(&mut self, index: usize, prev: bool) {
        let other = index ^ 1;
        if prev {
            let loop_prev = self.intersections[other].next == Some(index);
            if loop_prev && self.could_take_prev(index) {
                let prev = self.intersections[index]
                    .prev
                    .expect("loop prev has a link");
                self.take(prev, false);
            }
        } else {
            let loop_next = self.intersections[index].next == Some(other);
            if loop_next && self.could_take_next(index) {
                self.take(index, true);
            }
        }
    }

    /// T-joints (`:2625-2645`): connect with long arches; loops are
    /// only allowed for a very long arc.
    pub(super) fn connect_t_joints(&mut self, arches: &[SupportArcCost]) {
        let cost_high = self.line_spacing * 2.0;
        let mut candidates = Vec::with_capacity(self.intersections.len() * 2);
        for index in 0..self.intersections.len() {
            if self.could_take_prev(index) {
                candidates.push((arches[index * 2].cost, index, true));
            }
            if self.could_take_next(index) {
                candidates.push((arches[index * 2 + 1].cost, index, false));
            }
        }
        candidates.sort_by(|left, right| right.0.partial_cmp(&left.0).expect("finite costs"));
        for (cost, index, prev) in candidates {
            self.take_t_joint(index, prev, cost, cost_high);
        }
    }

    fn take_t_joint(&mut self, index: usize, prev: bool, cost: f64, cost_high: f64) {
        if prev {
            self.take_t_joint_prev(index, cost, cost_high);
        } else {
            self.take_t_joint_next(index, cost, cost_high);
        }
    }

    fn take_t_joint_prev(&mut self, index: usize, cost: f64, cost_high: f64) {
        if !self.could_take_prev(index) {
            return;
        }
        let Some(prev_index) = self.intersections[index].prev else {
            return;
        };
        if !self.same_chain(index, prev_index) || cost > cost_high {
            self.take(prev_index, false);
        }
    }

    fn take_t_joint_next(&mut self, index: usize, cost: f64, cost_high: f64) {
        if !self.could_take_next(index) {
            return;
        }
        let Some(next_index) = self.intersections[index].next else {
            return;
        };
        if !self.same_chain(index, next_index) || cost > cost_high {
            self.take(index, true);
        }
    }
}

/// `vertical(dir)`: the neighbour sits on the same vertical infill
/// line (`FillBase.cpp:1342`: same x in the connect frame, where
/// the infill lines are strictly vertical — i.e. the arch is
/// PARALLEL to the scan lines). Ares connects in the same rotated
/// working frame (scan lines vertical, constant x'), so the
/// predicate is x-equality on the connect-frame contour points.
/// Note: this detects the boundary walks parallel to the fill
/// (family B); the perpendicular zig-zag connector arches (same y')
/// are left for the zig-zag/T-joint phases — flipping the predicate
/// to y-equality consumes the connectors first and breaks the
/// serpent (the 14-pair-chains divergence).
fn vertical_dir(
    boundary: &[BoundaryContour],
    intersections: &[Intersection],
    index: usize,
    neighbor: usize,
) -> bool {
    let contour = &boundary[intersections[index]
        .contour_index
        .expect("connected intersection")];
    contour.points[intersections[index].point_index].x()
        == contour.points[intersections[neighbor].point_index].x()
}

/// `take_vertical_prev` (`:2353-2359`): prefer the untrimmed side,
/// else the longer contour.
fn take_vertical_prev(intersections: &[Intersection], index: usize) -> bool {
    if intersections[index].prev_trimmed == intersections[index].next_trimmed {
        intersections[index].not_taken_prev > intersections[index].not_taken_next
    } else {
        !intersections[index].prev_trimmed && intersections[index].next_trimmed
    }
}
