use ares_core::{GenerationMetadata, SliceError, SlicerContext};
use serde_json::json;

mod enum_declarations_support;

async fn rejects_state(key: &str, value: &str) {
    let work = enum_declarations_support::artifacts(&format!("{key}-{}", value.replace(' ', "_")));
    let input = enum_declarations_support::input(key, value);
    std::fs::write(work.join("input.3mf"), &input).unwrap();
    let project = ares_core::load_project(&input).unwrap();
    let loaded = enum_declarations_support::loaded_value(&project, key);
    let substitutions = ares_core::project_config_substitutions(&input).unwrap();
    let result =
        ares_core::slice_project(&input, GenerationMetadata::default(), SlicerContext::new(1))
            .await;
    let outcome = match &result {
        Ok(bytes) => {
            std::fs::write(work.join("output.gcode"), bytes).unwrap();
            json!({"kind": "success", "bytes": bytes.len()})
        }
        Err(error) => {
            json!({"kind": "error", "display": error.to_string(), "debug": format!("{error:?}")})
        }
    };
    std::fs::write(
        work.join("result.json"),
        serde_json::to_vec_pretty(&json!({
            "key": key, "authored": value, "loaded": loaded,
            "substitutions": substitutions.iter().map(|entry| json!({
                "key": entry.key, "replacement": entry.new_value,
            })).collect::<Vec<_>>(),
            "outcome": outcome,
        }))
        .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(&result, Err(SliceError::UnsupportedProjectFeature(detail))
            if detail.contains(key) && detail.contains(value)),
        "{key}={value}: {outcome}"
    );
    assert_eq!(loaded, value);
    assert!(!substitutions.iter().any(|entry| entry.key == key));
}

macro_rules! unsupported_states {
    ($($name:ident: $key:literal, $value:literal;)*) => {$(
        #[tokio::test]
        async fn $name() {
            rejects_state($key, $value).await;
        }
    )*};
}

unsupported_states! {
    infill_spiral_inset: "top_surface_pattern", "spiralinset";
    support_spiral_inset: "support_interface_pattern", "spiralinset";
    leading_edge_brim: "brim_type", "leading_edge_only";
    best_of_order: "print_order", "best_of";
    snake_order: "print_order", "snake";
    ultimaker_host: "host_type", "ultimaker";
    nozzle_manual: "filament_map_mode", "Nozzle Manual";
}
