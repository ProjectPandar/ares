use std::collections::BTreeMap;

use serde_json::Value;

use crate::{
    PerimeterOptions, PrintPathRole, SeamPosition, SliceError, SliceOptions, WallGenerator,
};

pub(super) fn parse_perimeter_options(
    options: &SliceOptions,
) -> Result<PerimeterOptions, SliceError> {
    let extrusion_options = options.extrusion_options()?;
    let external_line_width = extrusion_options.width_for_role(PrintPathRole::ExternalPerimeter);
    let min_nozzle_diameter = options
        .nozzle_diameters()?
        .into_iter()
        .fold(f64::INFINITY, f64::min);
    Ok(PerimeterOptions::new(
        options.non_negative_u32("wall_loops", 2)?,
        external_line_width,
        extrusion_options.width_for_role(PrintPathRole::InternalPerimeter),
        super::wall_direction::parse_wall_direction(options.values())?,
        super::wall_sequence::parse_wall_sequence(options.values())?,
    )
    .with_wall_generator(parse_wall_generator(options.values())?)
    .with_min_nozzle_diameter(min_nozzle_diameter)
    .with_wall_transition_length_percent(crate::options::parsing::parse_range_f64(
        "wall_transition_length",
        options.values().get("wall_transition_length"),
        100.0,
        0.0,
        f64::INFINITY,
    )?)
    .with_wall_transition_filter_deviation_percent(crate::options::parsing::parse_range_f64(
        "wall_transition_filter_deviation",
        options.values().get("wall_transition_filter_deviation"),
        25.0,
        0.0,
        f64::INFINITY,
    )?)
    .with_wall_transition_angle_degrees(crate::options::parsing::parse_range_f64(
        "wall_transition_angle",
        options.values().get("wall_transition_angle"),
        10.0,
        1.0,
        59.0,
    )?)
    .with_wall_distribution_count(parse_wall_distribution_count(options.values())?)
    .with_min_feature_size_percent(crate::options::parsing::parse_range_f64(
        "min_feature_size",
        options.values().get("min_feature_size"),
        25.0,
        0.0,
        f64::INFINITY,
    )?)
    .with_initial_layer_min_bead_width_percent(crate::options::parsing::parse_range_f64(
        "initial_layer_min_bead_width",
        options.values().get("initial_layer_min_bead_width"),
        85.0,
        0.0,
        f64::INFINITY,
    )?)
    .with_min_bead_width_percent(crate::options::parsing::parse_range_f64(
        "min_bead_width",
        options.values().get("min_bead_width"),
        85.0,
        0.0,
        f64::INFINITY,
    )?)
    .with_wall_maximum_resolution_mm(crate::options::parsing::parse_range_f64(
        "wall_maximum_resolution",
        options.values().get("wall_maximum_resolution"),
        0.5,
        0.005,
        0.5,
    )?)
    .with_wall_maximum_deviation_mm(crate::options::parsing::parse_range_f64(
        "wall_maximum_deviation",
        options.values().get("wall_maximum_deviation"),
        0.025,
        0.005,
        0.05,
    )?)
    .with_only_one_wall_first_layer(options.bool_option("only_one_wall_first_layer", false)?)
    .with_only_one_wall_top(options.bool_option("only_one_wall_top", false)?)
    .with_alternate_extra_wall(options.bool_option("alternate_extra_wall", false)?)
    .with_sparse_infill_density_percent(options.percent("sparse_infill_density", 20.0)?)
    .with_precise_outer_wall(options.bool_option("precise_outer_wall", true)?)
    .with_layer_height_mm(options.layer_height()?)
    .with_detect_overhang_wall(options.bool_option("detect_overhang_wall", true)?)
    .with_extra_perimeters_on_overhangs(
        options.bool_option("extra_perimeters_on_overhangs", false)?,
    )
    .with_overhang_reverse(options.bool_option("overhang_reverse", false)?)
    .with_overhang_reverse_internal_only(
        options.bool_option("overhang_reverse_internal_only", false)?,
    )
    .with_overhang_reverse_threshold_mm(parse_overhang_reverse_threshold(
        options.values(),
        external_line_width,
    )?)
    .with_make_overhang_printable(options.bool_option("make_overhang_printable", false)?)
    .with_make_overhang_printable_angle_degrees(options.range_f64(
        "make_overhang_printable_angle",
        55.0,
        0.0,
        90.0,
    )?)
    .with_make_overhang_printable_hole_size_mm2(options.range_f64(
        "make_overhang_printable_hole_size",
        0.0,
        0.0,
        f64::INFINITY,
    )?)
    .with_seam_position(parse_seam_position(options.values())?)
    .with_staggered_inner_seams(options.bool_option("staggered_inner_seams", false)?)
    .with_seam_gap_mm(parse_seam_gap(options.values(), external_line_width)?)
    .with_min_length_factor(crate::options::parsing::parse_range_f64(
        "min_length_factor",
        options.values().get("min_length_factor"),
        0.5,
        0.0,
        25.0,
    )?)
    .with_detect_thin_wall(options.bool_option("detect_thin_wall", false)?)
    .with_fuzzy_skin(crate::perimeters::FuzzySkinConfig::parse(
        options.values(),
        options.bool_option("fuzzy_skin_first_layer", false)?,
    )?))
}

fn parse_wall_generator(values: &BTreeMap<String, Value>) -> Result<WallGenerator, SliceError> {
    let Some(value) = values.get("wall_generator") else {
        return Ok(WallGenerator::Arachne);
    };
    match value.as_str() {
        Some("classic") => Ok(WallGenerator::Classic),
        Some("arachne") => Ok(WallGenerator::Arachne),
        _ => Err(SliceError::InvalidInput(
            "wall_generator must be classic or arachne".to_owned(),
        )),
    }
}

fn parse_wall_distribution_count(values: &BTreeMap<String, Value>) -> Result<u32, SliceError> {
    let Some(value) = values.get("wall_distribution_count") else {
        return Ok(1);
    };
    let value = match value {
        Value::Number(number) => number.as_f64(),
        Value::String(text) => text.parse().ok(),
        _ => None,
    }
    .ok_or_else(|| {
        SliceError::InvalidInput("wall_distribution_count must be an integer".to_owned())
    })?;
    if value.is_finite() && value.fract() == 0.0 && value >= 1.0 && value <= i32::MAX as f64 {
        Ok(value as u32)
    } else {
        Err(SliceError::InvalidInput(
            "wall_distribution_count is out of range".to_owned(),
        ))
    }
}

fn parse_seam_position(values: &BTreeMap<String, Value>) -> Result<SeamPosition, SliceError> {
    let Some(value) = values.get("seam_position") else {
        return Ok(SeamPosition::Aligned);
    };
    match value.as_str() {
        Some("nearest") => Ok(SeamPosition::Nearest),
        Some("aligned") => Ok(SeamPosition::Aligned),
        Some("aligned_back") => Ok(SeamPosition::AlignedBack),
        Some("back") => Ok(SeamPosition::Back),
        Some("random") => Ok(SeamPosition::Random),
        _ => Err(SliceError::InvalidInput(
            "seam_position must be nearest, aligned, aligned_back, back, or random".to_owned(),
        )),
    }
}

fn parse_seam_gap(
    values: &BTreeMap<String, Value>,
    external_line_width: f64,
) -> Result<f64, SliceError> {
    match values.get("seam_gap") {
        Some(value) => crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
            "seam_gap",
            value,
            external_line_width,
        ),
        None => Ok(external_line_width * 0.1),
    }
}

fn parse_overhang_reverse_threshold(
    values: &BTreeMap<String, Value>,
    external_line_width: f64,
) -> Result<f64, SliceError> {
    let Some(value) = values.get("overhang_reverse_threshold") else {
        return Ok(external_line_width * 0.5);
    };
    let threshold = crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
        "overhang_reverse_threshold",
        value,
        external_line_width,
    )?;
    if threshold <= 20.0 {
        Ok(threshold)
    } else {
        Err(SliceError::InvalidInput(
            "overhang_reverse_threshold is out of range".to_owned(),
        ))
    }
}
