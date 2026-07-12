use crate::{BrimType, SliceError};

pub(super) fn parse_brim_type(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<BrimType, SliceError> {
    let Some(value) = values.get("brim_type") else {
        return Ok(BrimType::AutoBrim);
    };
    match value.as_str() {
        Some("auto_brim") => Ok(BrimType::AutoBrim),
        Some("brim_ears") => Ok(BrimType::BrimEars),
        Some("painted") => Ok(BrimType::Painted),
        Some("outer_only") => Ok(BrimType::OuterOnly),
        Some("inner_only") => Ok(BrimType::InnerOnly),
        Some("outer_and_inner") => Ok(BrimType::OuterAndInner),
        Some("no_brim") => Ok(BrimType::NoBrim),
        _ => Err(SliceError::InvalidInput(
            "brim_type must be a valid Orca brim type".to_owned(),
        )),
    }
}

pub(super) fn parse_brim_ears_max_angle(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<f64, SliceError> {
    crate::options::parsing::parse_range_f64(
        "brim_ears_max_angle",
        values.get("brim_ears_max_angle"),
        125.0,
        0.0,
        180.0,
    )
}

pub(super) fn parse_brim_ears_detection_length(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<f64, SliceError> {
    crate::options::parsing::parse_range_f64(
        "brim_ears_detection_length",
        values.get("brim_ears_detection_length"),
        1.0,
        0.0,
        f64::INFINITY,
    )
}

pub(super) fn parse_efc_outline_offset(
    options: &crate::SliceOptions,
) -> Result<Option<f64>, SliceError> {
    let enabled = options.bool_option("brim_use_efc_outline", false)?;
    let compensation = options.range_f64("elefant_foot_compensation", 0.0, 0.0, f64::INFINITY)?;
    let layers = options.non_negative_u32("elefant_foot_compensation_layers", 1)?;
    let has_raft = options.raft_options()?.has_raft();
    if enabled && compensation > 0.0 && layers > 0 && !has_raft {
        Ok(Some(compensation))
    } else {
        Ok(None)
    }
}
