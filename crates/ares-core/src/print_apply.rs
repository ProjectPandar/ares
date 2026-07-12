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

#[derive(Clone, Copy, Debug, PartialEq)]
struct LayerConfigRangeInput {
    start: f64,
    end: f64,
    config_id: usize,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct NormalizedLayerRange {
    start: f64,
    end: f64,
    config_id: Option<usize>,
}

const ORCA_EPSILON: f64 = 1e-4;

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M272 stages PrintApply LayerRanges::assign before model-object apply wiring"
    )
)]
fn normalize_layer_ranges(input: &[LayerConfigRangeInput]) -> Vec<NormalizedLayerRange> {
    let mut ranges = Vec::with_capacity(input.len().saturating_add(1));
    let mut last_z = 0.0;

    for range in input {
        if range.end > last_z {
            let min_z = range.start.max(0.0);
            if min_z > last_z + ORCA_EPSILON {
                ranges.push(NormalizedLayerRange {
                    start: last_z,
                    end: min_z,
                    config_id: None,
                });
                last_z = min_z;
            }
            if range.end > last_z + ORCA_EPSILON {
                ranges.push(NormalizedLayerRange {
                    start: last_z,
                    end: range.end,
                    config_id: Some(range.config_id),
                });
                last_z = range.end;
            }
        }
    }

    if ranges.is_empty() {
        ranges.push(NormalizedLayerRange {
            start: 0.0,
            end: f64::MAX,
            config_id: None,
        });
    } else if ranges.last().is_some_and(|range| range.config_id.is_none()) {
        ranges.last_mut().expect("range exists").end = f64::MAX;
    } else {
        ranges.push(NormalizedLayerRange {
            start: ranges.last().expect("range exists").end,
            end: f64::MAX,
            config_id: None,
        });
    }

    ranges
}

struct PrintableFilamentGeometryOps<D, A, I> {
    diff: D,
    all_intersection: A,
    intersection: I,
}

#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "M274 stages PrintApply LayerRanges::config before model-object apply wiring"
    )
)]
fn layer_range_config_id(
    ranges: &[NormalizedLayerRange],
    requested: (f64, f64),
) -> Option<Option<usize>> {
    let key = (requested.0 - ORCA_EPSILON, requested.1 - ORCA_EPSILON);
    let found = ranges
        .iter()
        .find(|range| (range.start, range.end) >= key)?;
    if (found.start - requested.0).abs() > ORCA_EPSILON
        || (found.end - requested.1).abs() > ORCA_EPSILON
    {
        return None;
    }
    Some(found.config_id)
}

include!("print_apply/staged_modules_legacy.rs");

#[cfg(test)]
mod tests;
