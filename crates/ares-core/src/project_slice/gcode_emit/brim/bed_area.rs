//! Model::setPrintSpeedTable bed geometry and Brim.cpp:636-686 subtraction.
use crate::{
    Point2d, SliceError,
    geometry::{
        CoordinateScale, ExPolygon, JoinType, Point, Polygon, difference_ex,
        difference_polygons_ex, difference_polygons_paths, offset_paths,
    },
    project_slice::{
        perimeters::classic::traversal::PreparedPostClassicTraversal, state::global_bed_points,
    },
};

use super::{brim_geometry_error, scaled_f64};

pub(super) fn plate_shift(traversal: &PreparedPostClassicTraversal, object_index: usize) -> Point {
    let scale = traversal.scale;
    let (x, y) = super::super::footprint::print_object_origin(traversal, object_index);
    Point::new(
        (x / scale.factor()).round() as i64,
        (y / scale.factor()).round() as i64,
    )
}

pub(super) fn clip(
    traversal: &PreparedPostClassicTraversal,
    object_index: usize,
    logical_filament: usize,
    (area, no_brim_area, tower_exclusions): (Vec<ExPolygon>, Vec<ExPolygon>, &[ExPolygon]),
    resolution: f64,
) -> Result<Vec<ExPolygon>, SliceError> {
    let shift = plate_shift(traversal, object_index);
    // The complete no-brim collection is already in plate coordinates.
    // Existing model-local storage ↔ global geometry, not a plate-origin transform.
    let area = translate(area, shift);
    let clipped = clip_plate(
        traversal,
        logical_filament,
        (area, no_brim_area),
        tower_exclusions,
        resolution,
    )?;
    Ok(translate(clipped, Point::new(-shift.x(), -shift.y())))
}

pub(super) fn clip_plate(
    traversal: &PreparedPostClassicTraversal,
    logical_filament: usize,
    (area, mut forbidden): (Vec<ExPolygon>, Vec<ExPolygon>),
    tower_exclusions: &[ExPolygon],
    resolution: f64,
) -> Result<Vec<ExPolygon>, SliceError> {
    // Brim.cpp:651–659: the tower exclusions are already in the plate frame.
    forbidden.extend_from_slice(tower_exclusions);
    let scale = traversal.scale;
    let printer = &traversal.project.settings().printer.remaining;
    let bed = if printer.extruder_printable_area.0.is_empty() {
        // Same printable-area coordinates as PrintVolume::new; O6 owns their frame.
        let mut bed = polygon(
            global_bed_points(&traversal.project, &printer.printable_area.0),
            scale,
        )?;
        if bed.area() < 0.0 {
            bed.reverse();
        }
        let excluded = printer
            .bed_exclude_area
            .0
            .as_chunks::<4>()
            .0
            .iter()
            .map(|points| polygon(global_bed_points(&traversal.project, points), scale))
            .collect::<Result<Vec<_>, _>>()?;
        // Model.cpp:3026 selects the first flat difference result.
        difference_polygons_paths(&[bed], &excluded)
            .map_err(brim_geometry_error)?
            .swap_remove(0)
    } else {
        // Print.cpp:getObjectExtruderMap / Brim.cpp:666: logical wall → physical slot.
        let slot = traversal.resolved.views.full.project.gcode.filament_map.0[logical_filament].0
            as usize
            - 1;
        polygon(
            global_bed_points(&traversal.project, &printer.extruder_printable_area.0[slot]),
            scale,
        )?
    };
    let bed = [bed];
    // Brim.cpp:668: scale_(30.), jtRound, SCALED_RESOLUTION, then nonempty front().
    let expanded = offset_paths(
        &bed,
        scaled_f64(scale, 30.0)? as f32,
        JoinType::Round,
        resolution,
    )
    .map_err(brim_geometry_error)?;
    let border = difference_polygons_ex(&expanded, &bed).map_err(brim_geometry_error)?;
    if let Some(first) = border.into_iter().next() {
        forbidden.push(first);
    }
    difference_ex(&area, &forbidden).map_err(brim_geometry_error)
}

fn polygon(
    points: impl Iterator<Item = Point2d>,
    scale: CoordinateScale,
) -> Result<Polygon, SliceError> {
    points
        .map(|point| {
            Ok(Point::new(
                scaled_f64(scale, point.x)? as i64,
                scaled_f64(scale, point.y)? as i64,
            ))
        })
        .collect::<Result<Vec<_>, _>>()
        .map(Polygon::new)
}

pub(super) fn translate(area: Vec<ExPolygon>, shift: Point) -> Vec<ExPolygon> {
    area.into_iter()
        .map(|area| {
            let (contour, holes) = area.into_parts();
            let translate_polygon = |polygon: Polygon| {
                let mut points = polygon.into_points();
                for point in &mut points {
                    *point = Point::new(point.x() + shift.x(), point.y() + shift.y());
                }
                Polygon::new(points)
            };
            ExPolygon::new(
                translate_polygon(contour),
                holes.into_iter().map(translate_polygon).collect(),
            )
        })
        .collect()
}
