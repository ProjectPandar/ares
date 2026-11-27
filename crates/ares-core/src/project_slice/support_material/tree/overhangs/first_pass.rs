//! `TreeSupport.cpp:815-910` per-layer overhang/sharp-tail/cantilever kernel.

use super::{SmartOverhangs, overlaps};
use crate::{
    geometry::{
        BoundingBox, ClipperError, ExPolygon, FillRule, JoinType, Point, Polygon, difference_ex,
        intersection_ex, intersection_polygons_paths, offset_expolygon, offset_expolygons,
        union_ex, union_expolygons,
    },
    project_slice::{
        belt::floor_context::BeltFloorContext, support_material::layer_input::to_polygons,
    },
};

impl SmartOverhangs<'_> {
    pub(in super::super) fn detect_layer(
        &mut self,
        layer_index: usize,
        floor: Option<(BeltFloorContext, f64)>,
        elapsed_seconds: impl FnOnce() -> f64,
    ) -> Result<(), ClipperError> {
        let automatic = self.automatic;
        let enforce_layers = self.object.enforce_support_layers.0 as usize;
        if !automatic && layer_index > enforce_layers {
            return Ok(());
        }
        if layer_index == 0 {
            self.initial_sharp_tails();
            return Ok(());
        }
        let previous = &self.layers[layer_index - 1];
        let lower_offset = if layer_index < enforce_layers {
            -0.15 * self.extrusion_width
        } else {
            f64::from(previous.height as f32) / self.threshold_radians.tan()
        };
        let belt_lower = floor
            .map(|floor| self.belt_lower(layer_index, floor))
            .transpose()?;
        let lower = offset_expolygons(
            belt_lower.as_deref().unwrap_or(&previous.extrudable),
            self.scale.scaled_delta(lower_offset) as f32,
            JoinType::Square,
            0.0,
        )?;
        self.layers[layer_index].raw_overhangs =
            difference_ex(&self.layers[layer_index].extrudable, &lower)?;
        if elapsed_seconds() > 30.0 || self.layers[layer_index].raw_overhangs.len() > 100 {
            self.detect_sharp_tails = false;
            self.remove_small_overhangs = false;
            return Ok(());
        }
        if automatic && self.detect_sharp_tails {
            self.floating_sharp_tails(layer_index, belt_lower.as_deref())?;
        }
        let cantilever_base = offset_expolygons(
            &lower,
            self.scale
                .scaled_delta((self.extrusion_width - lower_offset).max(0.0) + 0.1)
                as f32,
            JoinType::Miter,
            3.0,
        )?;
        self.detect_cantilevers(layer_index, &cantilever_base)
    }

    // TreeSupport.cpp:925–986 uses the belt at the current layer's bottom.
    fn belt_lower(
        &self,
        layer_index: usize,
        (floor, print_z): (BeltFloorContext, f64),
    ) -> Result<Vec<ExPolygon>, ClipperError> {
        let previous = &self.layers[layer_index - 1];
        let current = &self.layers[layer_index];
        let shift = floor.lower_layer_shift(previous.height, self.scale);
        let translate = |polygon: &Polygon| {
            Polygon::new(
                polygon
                    .points()
                    .iter()
                    .map(|point| Point::new(point.x() + shift.x(), point.y() + shift.y()))
                    .collect(),
            )
        };
        let shifted: Vec<_> = previous
            .extrudable
            .iter()
            .map(|region| {
                ExPolygon::new(
                    translate(region.contour()),
                    region.holes().iter().map(translate).collect(),
                )
            })
            .collect();
        let lower = BoundingBox::from_expolygons(&previous.extrudable);
        let mut bounds = BoundingBox::from_expolygons(&current.extrudable);
        if let Some(lower) = lower {
            if let Some(bounds) = &mut bounds {
                bounds.merge_point(lower.min());
                bounds.merge_point(lower.max());
            } else {
                bounds = Some(lower);
            }
        }
        let Some(bounds) = bounds else {
            return Ok(shifted);
        };
        let bounds = bounds.inflated(self.scale.scaled_delta(10.0) as i64);
        let (min, max) = (bounds.min(), bounds.max());
        let window = Polygon::new(vec![
            min,
            Point::new(max.x(), min.y()),
            max,
            Point::new(min.x(), max.y()),
        ]);
        let belt = floor.half_plane(print_z - current.height, self.scale, true);
        let belt = intersection_polygons_paths(&belt, &[window])?;
        if belt.is_empty() {
            return Ok(shifted);
        }
        union_expolygons(&shifted, &union_ex(&belt, FillRule::NonZero)?)
    }

    fn initial_sharp_tails(&mut self) {
        let layer = &mut self.layers[0];
        let threshold = self.scale.scaled_delta(6.0);
        for polygon in &layer.extrudable {
            let bounds =
                BoundingBox::from_expolygon(polygon).expect("extrudable contour is nonempty");
            let width = (bounds.max().x() - bounds.min().x()) as f64;
            let height = (bounds.max().y() - bounds.min().y()) as f64;
            if !(width > threshold && height > threshold) {
                layer.sharp_tails.push(polygon.clone());
                layer.sharp_tail_heights.push(layer.height as f32);
            }
        }
    }

    fn floating_sharp_tails(
        &mut self,
        layer_index: usize,
        belt_lower: Option<&[ExPolygon]>,
    ) -> Result<(), ClipperError> {
        let (below, above) = self.layers.split_at_mut(layer_index);
        let current = &mut above[0];
        let lower = belt_lower.unwrap_or(&below[layer_index - 1].extrudable);
        let distance = 0.1 * self.extrusion_width_scaled;
        for polygon in &current.extrudable {
            let expanded = offset_expolygon(polygon, distance as f32, JoinType::Miter, 3.0)?;
            if overlaps(&expanded, lower)? {
                continue;
            }
            let eroded = offset_expolygon(polygon, (-distance) as f32, JoinType::Miter, 3.0)?;
            if !eroded.is_empty() {
                current.sharp_tails.push(polygon.clone());
                current.sharp_tail_heights.push(0.0);
                self.has_sharp_tails = true;
            }
        }
        Ok(())
    }

    fn detect_cantilevers(
        &mut self,
        layer_index: usize,
        base: &[crate::geometry::ExPolygon],
    ) -> Result<(), ClipperError> {
        let layer = &mut self.layers[layer_index];
        for polygon in &layer.raw_overhangs {
            let boundary = to_polygons(&intersection_ex(std::slice::from_ref(polygon), base)?);
            if boundary.is_empty() {
                continue;
            }
            let mut maximum: f64 = 0.0;
            for point in polygon.contour().points() {
                maximum = maximum.max(nearest_vertex_distance(*point, &boundary));
            }
            if maximum > self.scale.scaled_delta(3.0) {
                self.max_cantilever_distance = self.max_cantilever_distance.max(maximum);
                layer.cantilevers.push(polygon.clone());
                self.has_cantilever = true;
            }
        }
        Ok(())
    }
}

fn nearest_vertex_distance(point: Point, polygons: &[Polygon]) -> f64 {
    polygons
        .iter()
        .flat_map(|polygon| polygon.points())
        .map(|vertex| {
            let dx = (vertex.x() - point.x()) as f64;
            let dy = (vertex.y() - point.y()) as f64;
            (dx * dx + dy * dy).sqrt()
        })
        .fold(f64::MAX, f64::min)
}
