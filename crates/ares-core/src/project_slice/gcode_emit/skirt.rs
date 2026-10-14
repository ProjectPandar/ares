//! Skirt generation and emission — rewrite slice of `Print.cpp:_make_skirt`
//! (2646-2990, single combined group) and `GCode.cpp::generate_skirt`
//! (4388-4451): convex hull of the first `skirt_height` layers, round-join
//! offsets at `skirt_distance` stepping by the flow spacing, loops reversed
//! to outermost-first (`Print.cpp:2985`), emitted per layer with the
//! `skirt_speed` override (`GCode.cpp:6599-6604`) and seam-gap clipped
//! loops split at the angle-derived start point
//! (`GCode.cpp:4334-4359`, `extrude_loop` seam gap).

use crate::SliceError;
use crate::geometry::{
    CoordinateScale, JoinType, Point, Polygon, offset_paths, simplify_closed_points,
};
use crate::project_slice::gcode_emit::motion::{self, EmitState, LayerGeometry};
use crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal;
use crate::project_slice::perimeters::flow::build_nonbridging_flow;

mod generate;
mod geometry;
mod groups;
mod occupied;
#[cfg(test)]
mod tests;

use crate::geometry::convex_hull;
pub(in crate::project_slice::gcode_emit) use geometry::split_at_nearest;
use geometry::{closed_length, find_start_point};

pub(super) struct SkirtPlan {
    /// Plate-frame closed loops, outermost first (`Print.cpp:2985`).
    loops: Vec<Vec<Point>>,
    /// `_make_skirt` retains this outline even when skirt_loops is zero.
    occupied_hull: Vec<Polygon>,
    /// First-layer flow values; later layers rescale `mm3_per_mm` by the
    /// actual layer height (`GCode.cpp:4413`).
    width: f32,
    /// Number of layers the skirt is printed on (`skirt_height`, or every
    /// layer when the draft shield makes it infinite).
    pub(super) layer_count: usize,
    /// Seam target angle in degrees (`skirt_start_angle`).
    start_angle_deg: f64,
    single_loop_draft_shield: bool,
    plate_origin: (f64, f64),
}

impl SkirtPlan {
    pub(super) fn extrusion_paths(&self) -> impl Iterator<Item = (&[Point], f32)> {
        self.loops
            .iter()
            .map(|points| (points.as_slice(), self.width))
    }

    pub(super) fn covered_points(&self) -> impl Iterator<Item = Point> + '_ {
        self.occupied_hull.iter().flat_map(Polygon::points).copied()
    }

    pub(super) fn covered_bounds(&self, scale: CoordinateScale) -> Option<(f64, f64, f64, f64)> {
        let mut bounds = None::<(f64, f64, f64, f64)>;
        for point in self.covered_points() {
            let x = scale.unscale(point.x());
            let y = scale.unscale(point.y());
            bounds = Some(match bounds {
                Some((min_x, min_y, max_x, max_y)) => {
                    (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                }
                None => (x, y, x, y),
            });
        }
        bounds
    }

    /// Emits the loops for `layer_index`, split at the seam point
    /// (`GCode.cpp:4334-4359`, `GCode.cpp:4435-4441`).
    pub(super) fn emit(
        &self,
        output: &mut Vec<u8>,
        layer: SkirtLayer,
        geometry: LayerGeometry<'_>,
        state: &mut EmitState,
    ) {
        if layer.index >= self.layer_count {
            return;
        }
        // Print.cpp:2634-2637 -> GCode.hpp:218 -> GCodeWriter.cpp:608,
        // 715-716,789,873 subtracts plate offset at the writer. Ares currently
        // distributes that writer offset among path emitters; skirt needs it too.
        motion::set_origin(state, (-self.plate_origin.0, -self.plate_origin.1));
        let height = layer.height_mm as f32;
        let rounded_rectangle = 1.0 - 0.25 * std::f64::consts::PI;
        let mm3_per_mm = f64::from(
            (layer.height_mm * (f64::from(self.width) - layer.height_mm * rounded_rectangle))
                as f32,
        );
        let flow = motion::SkirtLoopFlow {
            width: self.width,
            height,
            mm3_per_mm,
        };
        let loops = self.loops_for_layer(layer.index);
        let mut seam_target = if layer.index == 0 {
            find_start_point(&loops[0], self.start_angle_deg)
        } else {
            state.last_scaled_position.map_or_else(
                || find_start_point(&loops[0], self.start_angle_deg),
                |(x, y)| Point::new(x, y),
            )
        };
        for loop_points in loops {
            let mut split = split_at_nearest(loop_points, seam_target)
                .into_iter()
                .map(|point| (point.x(), point.y()))
                .collect::<Vec<_>>();
            // `extrude_loop` clips the seam gap on the plate-frame
            // polyline (`GCode.cpp:5804` `clip_end` before
            // `point_to_gcode` applies `m_origin`); the final endpoint
            // truncates toward zero (`Polyline.cpp:52-72` `.cast`), which
            // differs from clipping after the local-frame translate.
            motion::clip::clip_end(&mut split, state.options.seam_gap / geometry.scale.factor());
            motion::emit_skirt_loop(
                output,
                split.into_iter(),
                motion::SkirtLoopParameters {
                    flow,
                    preclipped: true,
                },
                geometry,
                state,
            );
            if let Some(last) = state.last_scaled_position {
                seam_target = Point::new(last.0, last.1);
            }
        }
    }

    fn loops_for_layer(&self, layer_index: usize) -> &[Vec<Point>] {
        if layer_index > 0 && self.single_loop_draft_shield {
            &self.loops[self.loops.len() - 1..]
        } else {
            &self.loops
        }
    }
}

/// Per-layer emission inputs.
pub(super) struct SkirtLayer {
    pub(super) index: usize,
    pub(super) height_mm: f64,
}

fn checked_scale(scale: CoordinateScale, millimeters: f64) -> Result<i64, SliceError> {
    scale.checked_scale(millimeters).ok_or_else(|| {
        SliceError::InvalidInput("skirt geometry is outside the supported range".to_owned())
    })
}
