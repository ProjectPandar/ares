use serde_json::Value;

use crate::SliceError;

pub(super) fn solid_line_width(
    options: &super::super::SliceOptions,
    nozzle_diameter: f64,
) -> Result<f64, SliceError> {
    let configured = options.extrusion_width("line_width", 0.0, nozzle_diameter)?;
    Ok(if configured == 0.0 {
        nozzle_diameter
    } else {
        configured
    })
}

pub(super) fn parse_infill_anchor_length(
    value: Option<&Value>,
    key: &str,
    default: f64,
    sparse_spacing: f64,
) -> Result<f64, SliceError> {
    match value {
        Some(value) => crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
            key,
            value,
            sparse_spacing,
        ),
        None => Ok(default),
    }
}

pub(crate) fn parse_infill_combination_max_layer_height(
    value: Option<&Value>,
    nozzle_diameter: f64,
) -> Result<f64, SliceError> {
    let parsed = match value {
        Some(value) => crate::options::parsing::parse_non_negative_numeric_or_percent_over_base(
            "infill_combination_max_layer_height",
            value,
            nozzle_diameter,
        )?,
        None => nozzle_diameter,
    };
    Ok(if parsed > 0.0 {
        parsed.min(nozzle_diameter)
    } else {
        nozzle_diameter
    })
}

pub(super) fn parse_elephant_foot_compensation_layers(
    options: &super::super::SliceOptions,
) -> Result<usize, SliceError> {
    let layers = options.non_negative_u32("elefant_foot_compensation_layers", 1)?;
    if layers == 0 {
        Err(SliceError::InvalidInput(
            "elefant_foot_compensation_layers must be positive".to_owned(),
        ))
    } else {
        Ok(layers as usize)
    }
}

pub(super) fn sparse_spacing(line_width: f64, sparse_density_percent: f64) -> f64 {
    if sparse_density_percent > 0.0 {
        line_width / (sparse_density_percent / 100.0)
    } else {
        line_width
    }
}
