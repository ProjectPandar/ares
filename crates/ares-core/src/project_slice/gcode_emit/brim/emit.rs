//! Brim emission + covered-hull queries (`BrimPlan::emit` /
//! `covered_hull` / `covered_bounds`).

use super::{BrimPlan, append_loops, brim_geometry_error, scaled_f32, scaled_f64};
use crate::{
    SliceError,
    geometry::{CoordinateScale, Point},
    project_slice::gcode_emit::motion::{self, EmitState, LayerGeometry},
};

impl BrimPlan {
    pub(in crate::project_slice::gcode_emit) fn covered_hull(&self) -> &[Point] {
        &self.covered_hull
    }

    pub(in crate::project_slice::gcode_emit) fn covered_bounds(
        &self,
        scale: CoordinateScale,
    ) -> Option<(f64, f64, f64, f64)> {
        let mut bounds = None::<(f64, f64, f64, f64)>;
        for point in self.paths.iter().flatten() {
            let x = scale.unscale(point.x());
            let y = scale.unscale(point.y());
            bounds = Some(match bounds {
                Some((min_x, min_y, max_x, max_y)) => {
                    (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                }
                None => (x, y, x, y),
            });
        }
        let padding = 0.5 * f64::from(self.spacing);
        bounds.map(|(min_x, min_y, max_x, max_y)| {
            (
                min_x - padding,
                min_y - padding,
                max_x + padding,
                max_y + padding,
            )
        })
    }

    pub(in crate::project_slice::gcode_emit) fn emit(
        &self,
        output: &mut Vec<u8>,
        geometry: LayerGeometry<'_>,
        state: &mut EmitState,
    ) {
        for path in &self.paths {
            // Upstream emits the brim from the ring's natural (anchored)
            // start: the layer-change travel targets the loop's first
            // point, and `split_at(last_pos)` at the arrival is then
            // nearly a no-op (arrival ≈ ring start). Pre-splitting at
            // the PRE-travel nozzle position picks the wrong corner.
            motion::emit_brim_loop(
                output,
                path.iter().map(|point| (point.x(), point.y())),
                motion::SkirtLoopFlow {
                    width: self.width,
                    height: self.height,
                    mm3_per_mm: self.mm3_per_mm,
                },
                geometry,
                state,
            );
        }
    }
}
