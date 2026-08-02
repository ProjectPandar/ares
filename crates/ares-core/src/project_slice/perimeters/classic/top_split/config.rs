use crate::{FloatOrPercent, SliceError};

use super::super::types::PreparedPostClassicPrelude;
use crate::project_slice::perimeters::types::Flow;

#[derive(Clone, Copy)]
pub(super) struct ValidatedTopSplitConfig {
    pub(super) wall_loops: i32,
    pub(super) only_one_wall_top: bool,
    pub(super) interface_shells: bool,
    pub(super) min_width_top_surface: f64,
    pub(super) sparse_infill_width: f64,
    pub(super) outer_nozzle_diameter: f64,
    pub(super) has_gap_fill: bool,
}

pub(super) fn validate_project(
    prepared: &PreparedPostClassicPrelude,
) -> Result<Vec<Vec<Option<ValidatedTopSplitConfig>>>, SliceError> {
    prepared
        .objects
        .iter()
        .map(|object| {
            let (source_object_index, _) = object.object.identity();
            let resolved = prepared
                .resolved
                .objects
                .iter()
                .find(|item| item.source_object_index == source_object_index)
                .expect("Classic predecessor must retain its resolved object");
            let inputs = object.object.as_parts().1;
            inputs
                .iter()
                .zip(&object.records)
                .map(|(input, prelude)| match (input, prelude) {
                    (Some(input), Some(prelude)) => {
                        let region = object.object.region_options(input);
                        if region.wall_loops.0 < 0 {
                            return Err(invalid("wall_loops"));
                        }
                        validate_non_negative("gap_infill_speed", region.gap_infill_speed.0)?;
                        let nozzle_index = region
                            .outer_wall_filament_id
                            .0
                            .checked_sub(1)
                            .and_then(|value| usize::try_from(value).ok())
                            .filter(|index| {
                                *index
                                    < prepared
                                        .resolved
                                        .views
                                        .full
                                        .project
                                        .print
                                        .nozzle_diameter
                                        .0
                                        .len()
                            })
                            .unwrap_or(0);
                        let nozzle = prepared
                            .resolved
                            .views
                            .full
                            .project
                            .print
                            .nozzle_diameter
                            .0
                            .get(nozzle_index)
                            .map(|value| value.0)
                            .ok_or_else(|| invalid("nozzle_diameter"))?;
                        validate_non_negative("nozzle_diameter", nozzle)?;

                        let perimeter_width_mm = prepared.scale.unscale(prelude.perimeter_width);
                        let min_width_mm = absolute(
                            "min_width_top_surface",
                            region.min_width_top_surface,
                            perimeter_width_mm,
                        )?;
                        let mut sparse_width_mm = absolute(
                            "sparse_infill_line_width",
                            region.sparse_infill_line_width,
                            nozzle,
                        )?;
                        if sparse_width_mm == 0.0 {
                            sparse_width_mm = Flow::auto_infill_width(nozzle);
                        }

                        Ok(Some(ValidatedTopSplitConfig {
                            wall_loops: region.wall_loops.0,
                            only_one_wall_top: region.only_one_wall_top.0,
                            interface_shells: resolved.object.interface_shells.0,
                            min_width_top_surface: scaled_option(
                                prepared.scale,
                                "min_width_top_surface",
                                min_width_mm,
                            )?,
                            sparse_infill_width: scaled_option(
                                prepared.scale,
                                "sparse_infill_line_width",
                                sparse_width_mm,
                            )?,
                            outer_nozzle_diameter: nozzle,
                            has_gap_fill: prelude.has_gap_fill,
                        }))
                    }
                    (None, None) => Ok(None),
                    _ => unreachable!("Task 22O.1 record slots must remain aligned"),
                })
                .collect()
        })
        .collect()
}

fn absolute(key: &str, value: FloatOrPercent, base: f64) -> Result<f64, SliceError> {
    let value = match value {
        FloatOrPercent::Float(value) => value,
        FloatOrPercent::Percent(percent) => base * percent.0 / 100.0,
    };
    validate_non_negative(key, value)?;
    Ok(value)
}

fn scaled_option(
    scale: crate::geometry::CoordinateScale,
    key: &str,
    value: f64,
) -> Result<f64, SliceError> {
    scale.checked_scale(value).ok_or_else(|| invalid(key))?;
    Ok(value / scale.factor())
}

fn validate_non_negative(key: &str, value: f64) -> Result<(), SliceError> {
    if value.is_finite() && value >= 0.0 {
        Ok(())
    } else {
        Err(invalid(key))
    }
}

fn invalid(key: &str) -> SliceError {
    SliceError::InvalidInput(format!("invalid Orca option {key}"))
}

#[cfg(test)]
mod tests;
