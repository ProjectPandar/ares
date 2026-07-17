use std::collections::BTreeSet;

use serde_json::Value;

use crate::{Point2, SliceError};

fn printable_filament_change_guard(
    new_full_config_values: &serde_json::Map<String, Value>,
    old_poly: &[Point2],
    new_poly: &[Point2],
) -> Result<bool, SliceError> {
    if old_poly == new_poly {
        return Ok(false);
    }
    if is_manual_filament_map_mode(new_full_config_values.get("filament_map_mode"))? {
        return Ok(false);
    }
    Ok(true)
}

fn is_manual_filament_map_mode(value: Option<&Value>) -> Result<bool, SliceError> {
    let Some(value) = value else {
        return Ok(false);
    };
    let Some(value) = value.as_str() else {
        return Err(SliceError::InvalidInput(
            "filament_map_mode must be a string".to_owned(),
        ));
    };
    Ok(value == "fmmManual" || value == "Manual")
}

#[derive(Debug, PartialEq)]
struct PrintableAreaPolygons {
    printable: Vec<Point2>,
    extruders: Vec<Vec<Point2>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
struct ScaledPoint {
    x: i64,
    y: i64,
}

#[derive(Debug, Eq, PartialEq)]
struct ScaledPrintableAreaPolygons {
    printable: Vec<ScaledPoint>,
    extruders: Vec<Vec<ScaledPoint>>,
}

fn printable_area_polygons(
    new_full_config_values: &serde_json::Map<String, Value>,
) -> Result<PrintableAreaPolygons, SliceError> {
    let printable = parse_point_array(
        new_full_config_values.get("printable_area"),
        "printable_area must be an array of [x,y] points",
    )?;
    let extruders = match new_full_config_values.get("extruder_printable_area") {
        Some(value) => value
            .as_array()
            .ok_or_else(|| {
                SliceError::InvalidInput(
                    "extruder_printable_area must be an array of point arrays".to_owned(),
                )
            })?
            .iter()
            .map(|value| {
                parse_point_array(
                    Some(value),
                    "extruder_printable_area must be an array of point arrays",
                )
            })
            .collect::<Result<Vec<_>, _>>()?,
        None => Vec::new(),
    };
    Ok(PrintableAreaPolygons {
        printable,
        extruders,
    })
}

fn parse_point_array(value: Option<&Value>, message: &str) -> Result<Vec<Point2>, SliceError> {
    value
        .and_then(Value::as_array)
        .ok_or_else(|| SliceError::InvalidInput(message.to_owned()))?
        .iter()
        .map(|value| parse_point(value, message))
        .collect()
}

fn parse_point(value: &Value, message: &str) -> Result<Point2, SliceError> {
    let values = value
        .as_array()
        .filter(|values| values.len() == 2)
        .ok_or_else(|| SliceError::InvalidInput(message.to_owned()))?;
    let x = values[0]
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| SliceError::InvalidInput(message.to_owned()))?;
    let y = values[1]
        .as_f64()
        .filter(|value| value.is_finite())
        .ok_or_else(|| SliceError::InvalidInput(message.to_owned()))?;
    Ok(Point2::new(x, y))
}

const ORCA_DEFAULT_SCALING_FACTOR: f64 = 0.000001;

fn scale_printable_area_polygons(polygons: &PrintableAreaPolygons) -> ScaledPrintableAreaPolygons {
    ScaledPrintableAreaPolygons {
        printable: scale_polygon(&polygons.printable),
        extruders: polygons
            .extruders
            .iter()
            .map(|polygon| scale_polygon(polygon))
            .collect(),
    }
}

fn scale_polygon(points: &[Point2]) -> Vec<ScaledPoint> {
    points
        .iter()
        .map(|point| ScaledPoint {
            x: scale_coord(point.x()),
            y: scale_coord(point.y()),
        })
        .collect()
}

fn scale_coord(value: f64) -> i64 {
    (value / ORCA_DEFAULT_SCALING_FACTOR).round() as i64
}

fn collect_extruder_diff_first_results<F>(
    polygons: &ScaledPrintableAreaPolygons,
    mut diff: F,
) -> Vec<Vec<ScaledPoint>>
where
    F: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>,
{
    let mut split_polys = Vec::new();
    for poly in &polygons.extruders {
        let mut res = diff(&polygons.printable, poly);
        if !res.is_empty() {
            split_polys.push(res.remove(0));
        }
    }
    split_polys
}

fn append_all_extruder_intersection_first_result<F>(
    polygons: &ScaledPrintableAreaPolygons,
    split_polys: &mut Vec<Vec<ScaledPoint>>,
    intersection: F,
) where
    F: FnOnce(&[Vec<ScaledPoint>], &[Vec<ScaledPoint>]) -> Vec<Vec<ScaledPoint>>,
{
    let subject = [polygons.printable.clone()];
    let mut all_extruder_polys = intersection(&subject, &polygons.extruders);
    if !all_extruder_polys.is_empty() {
        split_polys.push(all_extruder_polys.remove(0));
    }
}

fn find_intersection_ids<F>(
    poly: &[ScaledPoint],
    contours: &[Vec<ScaledPoint>],
    mut intersection: F,
) -> BTreeSet<usize>
where
    F: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>,
{
    let mut result = BTreeSet::new();
    for (index, contour) in contours.iter().enumerate() {
        if !intersection(poly, contour).is_empty() {
            result.insert(index);
        }
    }
    result
}

fn printable_filament_intersection_ids_changed<F>(
    old_poly: &[ScaledPoint],
    new_poly: &[ScaledPoint],
    split_polys: &[Vec<ScaledPoint>],
    mut intersection: F,
) -> bool
where
    F: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>,
{
    let old_ids = find_intersection_ids(old_poly, split_polys, &mut intersection);
    let new_ids = find_intersection_ids(new_poly, split_polys, intersection);
    old_ids != new_ids
}

fn printable_filament_changed_staged<D, A, I>(
    new_full_config_values: &serde_json::Map<String, Value>,
    polys: (&[Point2], &[Point2]),
    diff: D,
    all_intersection: A,
    intersection: I,
) -> Result<bool, SliceError>
where
    D: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>,
    A: FnOnce(&[Vec<ScaledPoint>], &[Vec<ScaledPoint>]) -> Vec<Vec<ScaledPoint>>,
    I: FnMut(&[ScaledPoint], &[ScaledPoint]) -> Vec<Vec<ScaledPoint>>,
{
    let (old_poly, new_poly) = polys;
    if !printable_filament_change_guard(new_full_config_values, old_poly, new_poly)? {
        return Ok(false);
    }

    let polygons = printable_area_polygons(new_full_config_values)?;
    let scaled = scale_printable_area_polygons(&polygons);
    let mut split_polys = collect_extruder_diff_first_results(&scaled, diff);
    append_all_extruder_intersection_first_result(&scaled, &mut split_polys, all_intersection);

    Ok(printable_filament_intersection_ids_changed(
        &scale_polygon(old_poly),
        &scale_polygon(new_poly),
        &split_polys,
        intersection,
    ))
}

struct PrintableFilamentGeometryOps<D, A, I> {
    diff: D,
    all_intersection: A,
    intersection: I,
}

#[cfg_attr(not(test), expect(dead_code, reason = "M351 staged before wiring"))]
mod apply_extruder_count_change_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M344 staged before wiring"))]
mod apply_print_diff_config_invalidation_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M343 staged before wiring"))]
mod apply_status_initial_diff_update_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M342 staged before wiring"))]
mod apply_print_diff_set_reassign_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M341 staged before wiring"))]
mod apply_manual_filament_map_same_map_prune_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M340 staged before wiring"))]
mod apply_manual_filament_map_setup_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M339 staged before wiring"))]
mod apply_auto_filament_map_diff_prune_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M338 staged before wiring"))]
mod apply_filament_map_auto_mode_gate_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M337 staged before wiring"))]
mod apply_filament_map_mode_guard_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M329 staged before wiring"))]
mod apply_scarf_joint_seam_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M328 staged before wiring"))]
mod apply_support_used_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M335 staged before wiring"))]
mod apply_filament_map_extraction_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M273 staged before wiring"))]
mod instance_sync_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M282 staged before wiring"))]
mod mesh_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M284 staged before wiring"))]
mod volume_cache_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M279 staged before wiring"))]
mod model_volume_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M280 staged before wiring"))]
mod transform_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M287 staged before wiring"))]
mod print_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M300 staged before wiring"))]
mod painted_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M306 staged before wiring"))]
mod fuzzy_painted_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M307 staged before wiring"))]
mod region_merge_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M288 staged before wiring"))]
mod verify_update_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M293 staged before wiring"))]
mod verify_update_config_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M275 staged before wiring"))]
mod model_object_status_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M277 stages PrintObjectStatus"))]
mod print_object_status_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M314 staged before wiring"))]
mod generate_regions_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M317 staged before wiring"))]
mod generate_model_part_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M318 staged before wiring"))]
mod generate_modifier_parent_scan_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M319 staged before wiring"))]
mod generate_modifier_changed_config_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M320 staged before wiring"))]
mod generate_modifier_unchanged_fallback_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M321 staged before wiring"))]
mod generate_painted_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M322 staged before wiring"))]
mod generate_painted_region_sort_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M323 staged before wiring"))]
mod generate_fuzzy_volume_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M324 staged before wiring"))]
mod generate_fuzzy_painted_region_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M345 staged before wiring"))]
mod apply_full_config_placeholder_entry_state;

#[cfg_attr(not(test), expect(dead_code, reason = "M325 staged before wiring"))]
mod generate_fuzzy_painted_region_sort_state;

#[cfg(test)]
mod tests;
