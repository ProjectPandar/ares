//! Polygon-only stepped Hybrid expansion from `TreeSupport.cpp:2330-2353`.

use super::areas::LayerAreas;
use crate::geometry::{
    ClipperError, CoordinateScale, ExPolygon, FillRule, JoinType, difference_polygons_paths,
    offset_expolygons_paths, offset_paths, union_ex,
};
use crate::project_slice::support_material::layer_input::to_polygons;
use crate::project_slice::support_material::overhangs::overlaps;

pub(super) struct HybridExpansion<'a> {
    pub(super) object_slices: &'a [ExPolygon],
    pub(super) expansion: f64,
    pub(super) gap_xy: f64,
    pub(super) first_layer_width_scaled: i64,
    pub(super) scale: CoordinateScale,
}

impl HybridExpansion<'_> {
    pub(super) fn apply(&self, areas: &mut LayerAreas) -> Result<(), ClipperError> {
        let inflation = self.scale.scaled_delta(self.expansion) as f32;
        let trimming = offset_expolygons_paths(
            self.object_slices,
            self.scale.scaled_delta(self.gap_xy) as f32,
            JoinType::Square,
            0.0,
        )?;
        let steps = ((inflation / self.first_layer_width_scaled as f32).ceil() as i32).max(5);
        let step = inflation / steps as f32;
        let mut expanded = Vec::new();
        for polygon in std::mem::take(&mut areas.base) {
            if !overlaps(std::slice::from_ref(&polygon), &areas.polygons)? {
                expanded.push(polygon);
                continue;
            }
            let mut paths = to_polygons(std::slice::from_ref(&polygon));
            for _ in 0..steps {
                paths = difference_polygons_paths(
                    &offset_paths(&paths, step, JoinType::Miter, 3.0)?,
                    &trimming,
                )?;
            }
            expanded.extend(union_ex(&paths, FillRule::NonZero)?);
        }
        areas.base = expanded;
        Ok(())
    }
}
