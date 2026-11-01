//! Brim emission + covered-hull queries (`BrimPlan::emit` /
//! `covered_hull` / `covered_bounds`).

use super::BrimPlan;
use crate::{
    geometry::{CoordinateScale, Point},
    project_slice::gcode_emit::motion::{self, EmitState, LayerGeometry},
};

impl BrimPlan {
    pub(in crate::project_slice::gcode_emit) fn occupied_origin(
        &self,
        traversal: &crate::project_slice::perimeters::classic::traversal::PreparedPostClassicTraversal,
        object_index: usize,
    ) -> (f64, f64) {
        if self.owner_collection {
            (0.0, 0.0)
        } else {
            super::super::footprint::print_object_origin(traversal, object_index)
        }
    }

    pub(in crate::project_slice::gcode_emit) fn covered_hull(&self) -> &[Point] {
        &self.covered_hull
    }

    pub(in crate::project_slice::gcode_emit) fn covered_bounds(
        &self,
        scale: CoordinateScale,
    ) -> Option<(f64, f64, f64, f64)> {
        let mut bounds = None::<(f64, f64, f64, f64)>;
        // Brim.cpp:930-935,945-956 retains each object's original area,
        // not the merged carrier's emission paths, for occupied bounds.
        let paths = if self.area.is_empty() {
            self.paths.as_slice()
        } else {
            std::slice::from_ref(&self.covered_hull)
        };
        for point in paths.iter().flatten() {
            let x = scale.unscale(point.x());
            let y = scale.unscale(point.y());
            bounds = Some(match bounds {
                Some((min_x, min_y, max_x, max_y)) => {
                    (min_x.min(x), min_y.min(y), max_x.max(x), max_y.max(y))
                }
                None => (x, y, x, y),
            });
        }
        let padding = if self.area.is_empty() {
            0.5 * f64::from(self.spacing)
        } else {
            0.0
        };
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
        copy: usize,
    ) -> Result<(), crate::SliceError> {
        let origin = state.origin;
        if self.global || self.owner_collection {
            motion::set_origin(state, (0.0, 0.0));
        }
        // GCode.cpp:6065-6079: an uncombined owner plan emits only the
        // current instance's paths (brimMapByInstance keyed
        // `ObjectInstanceID`), never the owner's merged set.
        let paths: &[Vec<Point>] = if self.owner_collection
            && let Some(instance) = self.instances.iter().find(|entry| entry.copy == copy)
        {
            &instance.paths
        } else {
            &self.paths
        };
        for path in paths {
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
            )?;
        }
        if self.global || self.owner_collection {
            motion::set_origin(state, origin);
        }
        Ok(())
    }
}
