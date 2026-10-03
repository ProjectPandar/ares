//! Sequential sharp-tail extension from `TreeSupport.cpp:917-983`.

use super::{OverhangLayer, SmartOverhangs, overlaps};
use crate::{
    ProcessSupportType,
    geometry::{
        ClipperError, CoordinateScale, ExPolygon, JoinType, difference_ex, intersection_ex,
        offset_expolygons,
    },
    project_slice::support_material::sharp_tail_propagation::extent_size,
};

impl SmartOverhangs<'_> {
    pub(in super::super) fn propagate_sharp_tails(&mut self) -> Result<(), ClipperError> {
        if !matches!(self.object.support_type, ProcessSupportType::TreeAuto)
            || !self.detect_sharp_tails
        {
            return Ok(());
        }
        for layer_index in 1..self.layers.len() {
            let (below, above) = self.layers.split_at_mut(layer_index);
            let lower = &below[layer_index - 1];
            let layer = &mut above[0];
            layer.extend_sharp_tails(lower, self.extrusion_width_scaled, self.scale)?;
        }
        Ok(())
    }
}

impl OverhangLayer {
    fn extend_sharp_tails(
        &mut self,
        lower: &Self,
        width_scaled: f64,
        scale: CoordinateScale,
    ) -> Result<(), ClipperError> {
        for polygon in &self.extrudable {
            let Some(height) =
                lower.propagated_height(polygon, (self.height, width_scaled), scale)?
            else {
                continue;
            };
            self.sharp_tails.push(polygon.clone());
            self.sharp_tail_heights.push(height);
        }
        Ok(())
    }

    fn propagated_height(
        &self,
        polygon: &ExPolygon,
        (height, width_scaled): (f64, f64),
        scale: CoordinateScale,
    ) -> Result<Option<f32>, ClipperError> {
        let supported = intersection_ex(std::slice::from_ref(polygon), &self.sharp_tails)?;
        if supported.is_empty() {
            return Ok(None);
        }
        let size = extent_size(&supported);
        let threshold = scale.scaled_delta(6.0);
        if size.x() as f64 > threshold && size.y() as f64 > threshold {
            return Ok(None);
        }
        let mut accumulated_height = height as f32;
        for (index, tail) in self.sharp_tails.iter().enumerate() {
            if overlaps(std::slice::from_ref(tail), std::slice::from_ref(polygon))? {
                accumulated_height += self.sharp_tail_heights[index];
                break;
            }
        }
        if accumulated_height > 16.0 {
            return Ok(None);
        }
        let overhangs = difference_ex(std::slice::from_ref(polygon), &self.sharp_tails)?;
        let new_size = extent_size(&overhangs);
        let lower_size = extent_size(&self.sharp_tails);
        // Point(double, double) rounds before conversion, unlike Point::new_scale.
        let growth = scale.scaled_delta(5.0).round() as crate::geometry::Coord;
        if new_size.x() - lower_size.x() > growth && new_size.y() - lower_size.y() > growth {
            return Ok(None);
        }
        if !offset_expolygons(
            &overhangs,
            (-5.0 * width_scaled) as f32,
            JoinType::Miter,
            3.0,
        )?
        .is_empty()
        {
            return Ok(None);
        }
        Ok(Some(accumulated_height))
    }
}
