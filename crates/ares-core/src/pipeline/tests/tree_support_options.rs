use super::*;

#[test]
fn pipeline_rejects_invalid_tree_support_options() {
    for (key, value) in [
        ("tree_support_branch_distance", json!(0.999)),
        ("tree_support_tip_diameter", json!(0.099)),
        ("tree_support_branch_diameter", json!(10.001)),
        ("tree_support_branch_angle", json!(60.001)),
        ("tree_support_branch_diameter_angle", json!(15.001)),
        ("tree_support_angle_slow", json!(9.999)),
        ("tree_support_wall_count", json!(3)),
        ("tree_support_auto_brim", json!("true")),
        ("tree_support_brim_width", json!(-0.001)),
        ("tree_support_branch_distance_organic", json!(0.999)),
        ("tree_support_top_rate", json!(35.001)),
        ("tree_support_branch_diameter_organic", json!("invalid")),
        ("tree_support_branch_angle_organic", json!(60.001)),
    ] {
        let options: SliceOptions = serde_json::from_value(json!({ key: value })).unwrap();
        let err = run_slicing_pipeline(b"not a model", &options).unwrap_err();

        assert!(matches!(err, SliceError::InvalidInput(_)));
        assert!(err.to_string().contains(key));
    }
}
