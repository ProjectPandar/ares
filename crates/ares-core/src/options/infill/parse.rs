use serde_json::Value;

use super::{
    InfillOptions, InfillWallBoundaryOptions, InfillWallOverlapOptions, InternalBridgeFilter,
    extra_solid, scalars, top_surface,
};
use crate::SliceError;

use super::patterns::{
    parse_bottom_surface_pattern, parse_infill_rotate_template,
    parse_internal_solid_infill_pattern, parse_sparse_infill_pattern,
    parse_sparse_infill_rotate_template, parse_top_surface_pattern,
};
use scalars::{
    parse_elephant_foot_compensation_layers, parse_infill_anchor_length,
    parse_infill_combination_max_layer_height, solid_line_width, sparse_spacing,
};

pub(in crate::options) fn parse_infill_options(
    options: &super::super::SliceOptions,
) -> Result<InfillOptions, SliceError> {
    let nozzle_diameter = options.nozzle_diameters()?[0];
    let extrusion_options = options.extrusion_options()?;
    let shell_layers = options.shell_layer_options()?;
    let spiral_mode = options.bool_option("spiral_mode", false)?;
    let sparse_density_percent = options.effective_sparse_infill_density_percent()?;
    let line_width = options.sparse_infill_line_width()?;
    let solid_line_width = solid_line_width(options, nozzle_diameter)?;
    let sparse_spacing = sparse_spacing(line_width, sparse_density_percent);
    let anchor = parse_infill_anchor_length(
        options.values().get("infill_anchor"),
        "infill_anchor",
        4.0 * sparse_spacing,
        sparse_spacing,
    )?;
    let anchor_max = parse_infill_anchor_length(
        options.values().get("infill_anchor_max"),
        "infill_anchor_max",
        20.0,
        sparse_spacing,
    )?;

    Ok(InfillOptions {
        sparse_density_percent,
        direction_degrees: options.range_f64("infill_direction", 45.0, 0.0, 360.0)?,
        sparse_infill_rotate_template_degrees: parse_sparse_infill_rotate_template(
            options.values().get("sparse_infill_rotate_template"),
        )?,
        line_width,
        fill_multiline: parse_fill_multiline(options)?,
        solid_line_width,
        minimum_sparse_infill_area_mm2: options.range_f64(
            "minimum_sparse_infill_area",
            15.0,
            0.0,
            f64::INFINITY,
        )?,
        pattern: parse_sparse_infill_pattern(options.values().get("sparse_infill_pattern"))?,
        solid_direction_degrees: options.range_f64("solid_infill_direction", 45.0, 0.0, 360.0)?,
        bridge_angle_degrees: options.range_f64("bridge_angle", 0.0, 0.0, f64::INFINITY)?,
        internal_bridge_angle_degrees: options.range_f64(
            "internal_bridge_angle",
            0.0,
            0.0,
            f64::INFINITY,
        )?,
        bridge_density_percent: options.range_f64("bridge_density", 100.0, 10.0, 120.0)?,
        internal_bridge_density_percent: options.range_f64(
            "internal_bridge_density",
            100.0,
            10.0,
            100.0,
        )?,
        internal_bridge_filter: InternalBridgeFilter::parse(
            options.values().get("dont_filter_internal_bridges"),
        )?,
        top_surface_density_percent: options.range_f64("top_surface_density", 100.0, 0.0, 100.0)?,
        min_width_top_surface_mm: top_surface::parse_min_width_top_surface(options)?,
        calib_flowrate_topinfill_special_order: options.bool_option(
            "calib_flowrate_topinfill_special_order",
            false,
        )?,
        bottom_surface_density_percent: options.range_f64(
            "bottom_surface_density",
            100.0,
            10.0,
            100.0,
        )?,
        elephant_foot_layers_density_percent: options.range_f64(
            "elefant_foot_layers_density",
            100.0,
            50.0,
            100.0,
        )?,
        elephant_foot_compensation_layers: parse_elephant_foot_compensation_layers(options)?,
        solid_infill_rotate_template_degrees: parse_infill_rotate_template(
            "solid_infill_rotate_template",
            options.values().get("solid_infill_rotate_template"),
        )?,
        internal_solid_infill_pattern: parse_internal_solid_infill_pattern(
            options.values().get("internal_solid_infill_pattern"),
        )?,
        bottom_surface_pattern: parse_bottom_surface_pattern(
            options.values().get("bottom_surface_pattern"),
        )?,
        top_surface_pattern: parse_top_surface_pattern(options.values().get("top_surface_pattern"))?,
        extra_solid_infills: extra_solid::ExtraSolidInfills::parse(
            options.values().get("extra_solid_infills"),
        )?,
        detect_narrow_internal_solid_infill: options.bool_option(
            "detect_narrow_internal_solid_infill",
            true,
        )?,
        shell_layers,
        spiral_mode,
        symmetric_infill_y_axis: options.bool_option("symmetric_infill_y_axis", false)?,
        infill_combination: options.bool_option("infill_combination", false)?,
        infill_combination_max_layer_height_mm: parse_infill_combination_max_layer_height(
            options.values().get("infill_combination_max_layer_height"),
            nozzle_diameter,
        )?,
        infill_anchor_length_mm: anchor.min(anchor_max),
        infill_shift_step_mm: options.range_f64("infill_shift_step", 0.4, 0.0, 10.0)?,
        wall_overlap: InfillWallOverlapOptions::parse(options)?,
        wall_boundary: InfillWallBoundaryOptions::parse(options, &extrusion_options)?,
    })
}

fn parse_fill_multiline(options: &super::super::SliceOptions) -> Result<usize, SliceError> {
    let Some(value) = options.values().get("fill_multiline") else {
        return Ok(1);
    };
    let value = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse::<f64>().ok(),
        _ => None,
    }
    .ok_or_else(|| SliceError::InvalidInput("fill_multiline must be an integer".to_owned()))?;
    if !(value.is_finite() && value.fract() == 0.0 && (1.0..=10.0).contains(&value)) {
        return Err(SliceError::InvalidInput(
            "fill_multiline must be in the range 1..=10".to_owned(),
        ));
    }
    Ok(value as usize)
}
