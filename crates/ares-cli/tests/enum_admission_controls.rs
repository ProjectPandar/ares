use serde_json::{Value, json};

mod enum_declarations_support;

#[test]
fn retained_states_roundtrip_without_substitution() {
    for (key, value) in [
        ("top_surface_pattern", "monotonicline"),
        ("support_interface_pattern", "auto"),
        ("brim_type", "auto_brim"),
        ("print_order", "default"),
        ("host_type", "octoprint"),
        ("filament_map_mode", "Auto For Flush"),
    ] {
        let input = enum_declarations_support::input(key, value);
        let project = ares_core::load_project(&input).unwrap();
        assert_eq!(
            enum_declarations_support::loaded_value(&project, key),
            value
        );
        assert!(
            !ares_core::project_config_substitutions(&input)
                .unwrap()
                .iter()
                .any(|entry| entry.key == key)
        );
    }
    let work = enum_declarations_support::artifacts("retained-states");
    std::fs::write(work.join("verified.json"), b"{\"verified\":true}\n").unwrap();
}

#[test]
fn unknown_tokens_and_gui_default_sentinel_keep_source_substitution() {
    let work = enum_declarations_support::artifacts("substitution-controls");
    let mut observations = Vec::new();
    for (key, token) in [
        ("top_surface_pattern", "not-a-pattern"),
        ("filament_map_mode", "Default"),
    ] {
        let input = enum_declarations_support::input(key, token);
        let project = ares_core::load_project(&input).unwrap();
        let substitutions = ares_core::project_config_substitutions(&input).unwrap();
        let actual = enum_declarations_support::loaded_value(&project, key);
        if token == "Default" {
            assert!(!substitutions.iter().any(|entry| entry.key == key));
            assert_eq!(actual, "Auto For Flush");
        } else if let Some(substitution) = substitutions.iter().find(|entry| entry.key == key) {
            assert_eq!(actual, substitution.new_value);
            assert_eq!(
                substitution.new_value,
                ares_core::option_definition(key).unwrap().default_value
            );
        } else {
            assert_eq!(
                actual,
                ares_core::option_definition(key).unwrap().default_value
            );
        }
        observations.push(json!({"key":key,"authored":token,"actual":actual}));
    }
    std::fs::write(
        work.join("verified.json"),
        serde_json::to_vec_pretty(&observations).unwrap(),
    )
    .unwrap();
}

#[test]
fn shared_infill_type_map_does_not_broaden_sparse_gui_and_wire_names() {
    let input = enum_declarations_support::input("sparse_infill_pattern", "spiralinset");
    let project = ares_core::load_project(&input).unwrap();
    assert_eq!(
        json!(project.settings().process.region.sparse_infill_pattern),
        "spiralinset"
    );
    assert!(
        !ares_core::project_config_substitutions(&input)
            .unwrap()
            .iter()
            .any(|entry| entry.key == "sparse_infill_pattern")
    );
    assert!(
        !ares_core::option_metadata("sparse_infill_pattern")
            .unwrap()
            .enum_values
            .contains(&"spiralinset")
    );
    for key in ["nozzle_type", "printer_structure"] {
        let decoded = ares_core::decode_config_value_by_key(key, "undefine").unwrap();
        assert_eq!(
            ares_core::encode_config_value_by_key(key, &decoded).unwrap(),
            "undefine"
        );
        let value: Value = serde_json::from_str(&decoded).unwrap();
        assert!(value == "undefine" || value == json!(["undefine"]));
    }
    let work = enum_declarations_support::artifacts("shared-map-controls");
    std::fs::write(work.join("verified.json"), b"{\"verified\":true}\n").unwrap();
}
