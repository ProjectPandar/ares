use super::super::*;
use serde_json::json;

#[test]
fn deserialization_does_not_normalize_spiral_mode_conflicts_automatically() {
    let options: SliceOptions = serde_json::from_value(json!({
        "spiral_mode": true,
        "wall_loops": 3,
        "alternate_extra_wall": true,
        "top_shell_layers": 4,
        "sparse_infill_density": 15,
    }))
    .unwrap();

    assert_eq!(options.values()["spiral_mode"], json!(true));
    assert_eq!(options.values()["wall_loops"], json!(3));
    assert_eq!(options.values()["alternate_extra_wall"], json!(true));
    assert_eq!(options.values()["top_shell_layers"], json!(4));
    assert_eq!(options.values()["sparse_infill_density"], json!(15));
    assert!(!options.values().contains_key("retract_when_changing_layer"));
    assert!(
        !options
            .values()
            .contains_key("filament_retract_when_changing_layer")
    );
}
