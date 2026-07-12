use super::super::*;
use crate::{DraftShield, SliceError};
use serde_json::json;

#[test]
fn parses_draft_shield_for_skirt_options() {
    let missing: SliceOptions = serde_json::from_value(json!({})).unwrap();
    assert_eq!(
        missing.skirt_options().unwrap().draft_shield(),
        DraftShield::Disabled
    );

    let disabled: SliceOptions = serde_json::from_value(json!({
        "draft_shield": "disabled"
    }))
    .unwrap();
    assert_eq!(
        disabled.skirt_options().unwrap().draft_shield(),
        DraftShield::Disabled
    );

    let enabled: SliceOptions = serde_json::from_value(json!({
        "draft_shield": "enabled"
    }))
    .unwrap();
    assert_eq!(
        enabled.skirt_options().unwrap().draft_shield(),
        DraftShield::Enabled
    );

    let invalid: SliceOptions = serde_json::from_value(json!({
        "draft_shield": "broken"
    }))
    .unwrap();
    assert!(matches!(
        invalid.skirt_options(),
        Err(SliceError::InvalidInput(message)) if message == "draft_shield must be disabled or enabled"
    ));
}
