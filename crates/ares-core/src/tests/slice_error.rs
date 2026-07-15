use crate::SliceError;

#[test]
fn slice_error_display_mappings_are_stable() {
    assert_eq!(SliceError::EmptyInput.to_string(), "slice input is empty");
    assert_eq!(
        SliceError::InvalidInput("invalid input sentinel".to_owned()).to_string(),
        "invalid input sentinel"
    );
    assert_eq!(
        SliceError::ProjectSlicingIncomplete.to_string(),
        "ProjectSlicingIncomplete"
    );
}

#[test]
fn unsupported_project_feature_display_is_stable() {
    let supplied_document = r#"{"filament_shrink":[95],"sentinel":"UNRELATED_DOCUMENT_SENTINEL"}"#;
    let display = SliceError::UnsupportedProjectFeature("filament_shrink".to_owned()).to_string();

    assert_eq!(display, "unsupported project feature: filament_shrink");
    assert!(!display.contains(supplied_document));
    assert!(!display.contains("UNRELATED_DOCUMENT_SENTINEL"));
}
