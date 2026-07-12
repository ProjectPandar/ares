use serde_json::Value;

use crate::{
    extrusions::SmallAreaInfillFlowCompensation,
    options::infill::patterns::{
        parse_bottom_surface_pattern, parse_internal_solid_infill_pattern,
        parse_top_surface_pattern,
    },
    InfillPattern, SliceError,
};

use super::SliceOptions;

pub(super) fn parse(
    options: &SliceOptions,
) -> Result<SmallAreaInfillFlowCompensation, SliceError> {
    if !options.bool_option("small_area_infill_flow_compensation", false)? {
        validate_model_entries(options.values().get("small_area_infill_flow_compensation_model"))?;
        return Ok(SmallAreaInfillFlowCompensation::disabled());
    }

    SmallAreaInfillFlowCompensation::parse(
        model_entries(options.values().get("small_area_infill_flow_compensation_model"))?,
        is_supported_pattern(parse_bottom_surface_pattern(
            options.values().get("bottom_surface_pattern"),
        )?),
        is_supported_pattern(parse_internal_solid_infill_pattern(
            options.values().get("internal_solid_infill_pattern"),
        )?),
        is_supported_pattern(parse_top_surface_pattern(
            options.values().get("top_surface_pattern"),
        )?),
    )
}

fn validate_model_entries(value: Option<&Value>) -> Result<(), SliceError> {
    if value.is_some() {
        SmallAreaInfillFlowCompensation::parse(model_entries(value)?, false, false, false)?;
    }
    Ok(())
}

pub(crate) fn model_entries(value: Option<&Value>) -> Result<Vec<String>, SliceError> {
    let Some(value) = value else {
        return Ok(SmallAreaInfillFlowCompensation::default_model_entries()
            .iter()
            .map(|entry| (*entry).to_owned())
            .collect());
    };

    match value {
        Value::Array(entries) => entries
            .iter()
            .map(|entry| {
                let Some(text) = entry.as_str() else {
                    return Err(SliceError::InvalidInput(
                        "small_area_infill_flow_compensation_model entries must be strings"
                            .to_owned(),
                    ));
                };
                let text = text.trim();
                if text.is_empty() {
                    return Err(SliceError::InvalidInput(
                        "small_area_infill_flow_compensation_model entries must be non-empty"
                            .to_owned(),
                    ));
                }
                Ok(text.to_owned())
            })
            .collect(),
        Value::String(text) => Ok(text
            .split(['\n', ';'])
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(str::to_owned)
            .collect()),
        _ => Err(SliceError::InvalidInput(
            "small_area_infill_flow_compensation_model must be a string or string list".to_owned(),
        )),
    }
}

const fn is_supported_pattern(pattern: InfillPattern) -> bool {
    matches!(
        pattern,
        InfillPattern::Rectilinear
            | InfillPattern::AlignedRectilinear
            | InfillPattern::Monotonic
            | InfillPattern::MonotonicLine
    )
}
