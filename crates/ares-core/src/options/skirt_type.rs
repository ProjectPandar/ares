use crate::{SkirtType, SliceError};

pub(super) fn parse_skirt_type(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<SkirtType, SliceError> {
    let Some(value) = values.get("skirt_type") else {
        return Ok(SkirtType::Combined);
    };
    match value.as_str() {
        Some("combined") => Ok(SkirtType::Combined),
        Some("perobject") => Ok(SkirtType::PerObject),
        _ => Err(SliceError::InvalidInput(
            "skirt_type must be combined or perobject".to_owned(),
        )),
    }
}
