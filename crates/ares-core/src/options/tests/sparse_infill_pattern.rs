use super::super::*;
use serde_json::json;

#[test]
fn sparse_infill_pattern_defaults_to_crosshatch_scaffold() {
    let infill = SliceOptions::default().infill_options().unwrap();

    assert_eq!(infill.pattern(), InfillPattern::CrossHatch);
}

#[test]
fn parses_supported_sparse_infill_patterns() {
    for (value, expected) in [
        ("rectilinear", InfillPattern::Rectilinear),
        ("alignedrectilinear", InfillPattern::AlignedRectilinear),
        ("line", InfillPattern::Line),
        ("grid", InfillPattern::Grid),
        ("crosshatch", InfillPattern::CrossHatch),
    ] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "sparse_infill_pattern": value })).unwrap();

        assert_eq!(options.infill_options().unwrap().pattern(), expected);
    }
}

#[test]
fn rejects_known_unimplemented_sparse_infill_patterns() {
    for value in ["gyroid", "honeycomb", "triangles", "cubic", "lightning"] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "sparse_infill_pattern": value })).unwrap();

        assert!(matches!(
            options.infill_options(),
            Err(SliceError::InvalidInput(_))
        ));
    }
}

#[test]
fn rejects_unknown_sparse_infill_pattern() {
    let options: SliceOptions =
        serde_json::from_value(json!({ "sparse_infill_pattern": "not-orca" })).unwrap();

    assert!(matches!(
        options.infill_options(),
        Err(SliceError::InvalidInput(_))
    ));
}

#[test]
fn rejects_monotonic_sparse_infill_patterns_until_sparse_engines_exist() {
    for value in ["monotonic", "monotonicline"] {
        let options: SliceOptions =
            serde_json::from_value(json!({ "sparse_infill_pattern": value })).unwrap();

        let err = options.infill_options().unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains("sparse_infill_pattern"));
    }
}
