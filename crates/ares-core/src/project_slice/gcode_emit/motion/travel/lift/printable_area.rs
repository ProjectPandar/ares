use super::EmitState;
use crate::{
    Point2d, ProjectSettings, SliceError,
    geometry::{Line, Point, Polygon, intersection_polygons_paths},
};

impl EmitState {
    // GCodeWriter.cpp:181–225: capture full polygons once at print-config application.
    pub(in crate::project_slice::gcode_emit) fn apply_printable_area(
        &mut self,
        settings: &ProjectSettings,
    ) -> Result<(), SliceError> {
        let printer = &settings.printer.remaining;
        self.bed_printable_area = (printer.printable_area.0.len() >= 3)
            .then(|| scaled_polygon(&printer.printable_area.0, self.scale_factor));
        self.extruder_printable_areas = printer
            .extruder_printable_area
            .0
            .iter()
            .map(|points| {
                extruder_area(points, self.bed_printable_area.as_ref(), self.scale_factor)
            })
            .collect::<Result<_, _>>()?;
        Ok(())
    }

    // GCodeWriter.cpp:229–260: the active writer filament owns the physical area.
    pub(super) fn spiral_lift_fits_printable_area(&self, center: (f64, f64), radius: f64) -> bool {
        let area = self
            .extruder_selected
            .then(|| self.extruders.active_physical())
            .and_then(|id| self.extruder_printable_areas.get(id))
            .filter(|area| area.points().len() >= 3)
            .or(self.bed_printable_area.as_ref());
        let Some(area) = area else { return true };
        let center = Point::new(
            (center.0 / self.scale_factor) as i64,
            (center.1 / self.scale_factor) as i64,
        );
        let scaled_radius = radius / self.scale_factor;
        let radius_squared = scaled_radius * scaled_radius;
        if !area.contains(&center) {
            return false;
        }
        let points = area.points();
        for i in 0..points.len() {
            if Line::new(points[i], points[(i + 1) % points.len()]).distance_to_squared(center)
                < radius_squared
            {
                return false;
            }
        }
        true
    }
}

fn scaled_polygon(points: &[Point2d], scale: f64) -> Polygon {
    let mut polygon = Polygon::new(
        points
            .iter()
            .map(|point| Point::new((point.x / scale) as i64, (point.y / scale) as i64))
            .collect(),
    );
    if !polygon.is_counter_clockwise() {
        polygon.reverse();
    }
    polygon
}

fn extruder_area(
    points: &[Point2d],
    bed: Option<&Polygon>,
    scale: f64,
) -> Result<Polygon, SliceError> {
    if points.len() < 3 {
        return Ok(bed.cloned().unwrap_or_else(|| Polygon::new(Vec::new())));
    }
    let polygon = scaled_polygon(points, scale);
    let Some(bed) = bed else { return Ok(polygon) };
    let clipped =
        intersection_polygons_paths(std::slice::from_ref(&polygon), std::slice::from_ref(bed))
            .map_err(|error| {
                SliceError::InvalidInput(format!(
                    "extruder printable-area intersection failed: {error:?}"
                ))
            })?;
    let mut largest = None;
    let mut best_area = 0.0;
    for contour in clipped {
        let area = contour.area().abs();
        if area > best_area {
            best_area = area;
            largest = Some(contour);
        }
    }
    Ok(largest.unwrap_or(polygon))
}
