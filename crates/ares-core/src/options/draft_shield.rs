use crate::{DraftShield, SliceError};

pub(super) fn parse_draft_shield(
    values: &std::collections::BTreeMap<String, serde_json::Value>,
) -> Result<DraftShield, SliceError> {
    let Some(value) = values.get("draft_shield") else {
        return Ok(DraftShield::Disabled);
    };
    match value.as_str() {
        Some("disabled") => Ok(DraftShield::Disabled),
        Some("enabled") => Ok(DraftShield::Enabled),
        _ => Err(SliceError::InvalidInput(
            "draft_shield must be disabled or enabled".to_owned(),
        )),
    }
}
