use serde_json::{Value, json};

use super::{input, support::Case};

#[tokio::test]
async fn equal_generated_name_reuses_the_loaded_candidate_without_inserting() {
    let catalog = input::catalog(
        &[],
        &[input::process(
            "Missing Child(selection.3mf)",
            "0.2",
            "user",
        )],
    );
    local_selection(
        "selection-collision-reuse",
        input::settings("Missing Child", "0.2"),
        catalog,
        ("Missing Child(selection.3mf)", false),
    )
    .await;
}

#[tokio::test]
async fn unequal_generated_names_progress_before_the_basename_suffix() {
    let catalog = input::catalog(
        &[],
        &[
            input::process("Missing Child(selection.3mf)", "0.1", "user"),
            input::process("Missing Child-1(selection.3mf)", "0.15", "user"),
        ],
    );
    local_selection(
        "selection-collision-progress",
        input::settings("Missing Child", "0.2"),
        catalog,
        ("Missing Child-2(selection.3mf)", true),
    )
    .await;
}

#[tokio::test]
async fn nameless_parentless_collisions_use_bare_decimal_prefixes() {
    let catalog = input::catalog(
        &[],
        &[
            input::process("(selection.3mf)", "0.1", "user"),
            input::process("1(selection.3mf)", "0.15", "user"),
        ],
    );
    local_selection(
        "selection-nameless-numbering",
        input::settings("", "0.2"),
        catalog,
        ("2(selection.3mf)", true),
    )
    .await;
}

#[tokio::test]
async fn nameless_selection_uses_the_unavailable_parent_as_its_prefix() {
    let mut settings = input::settings("", "0.2");
    settings["inherits_group"] = json!(["Missing Parent", "", ""]);
    local_selection(
        "selection-nameless-parent-prefix",
        settings,
        input::catalog(&[], &[]),
        ("Missing Parent(selection.3mf)", true),
    )
    .await;
}

async fn local_selection(label: &str, settings: Value, catalog: Value, expected: (&str, bool)) {
    let Some(case) = Case::with_catalog(label, settings, catalog) else {
        return;
    };
    let (name, created) = expected;
    let native = &case.native["projects"][0]["result"]["process"];
    assert_eq!(native["selected"]["name"], name);
    assert_eq!(native["selected"]["system"], false);
    assert_eq!(native["selected"]["external"], created);
    assert_eq!(native["selected"]["project_local"], created);
    assert_eq!(native["dirty"], false);
    assert_eq!(native["dirty_options"], json!([]));
    let before = case.native["before"]["process"]["records"]
        .as_array()
        .unwrap();
    let after = native["records"].as_array().unwrap();
    assert_eq!(after.len(), before.len() + usize::from(created));
    for record in before {
        let mut expected_record = record.clone();
        if record["default"] == true && created {
            expected_record["visible"] = json!(false);
        }
        assert_eq!(
            after
                .iter()
                .find(|preset| preset["name"] == record["name"])
                .unwrap(),
            &expected_record
        );
    }
    case.write(
        "native-premise.json",
        &json!({"passed": true, "created": created, "name": name}),
    );
    let cli = case.cli().await.unwrap();
    assert_eq!(cli["layer_height"], "0.2");
    case.write(
        "cli-premise.json",
        &json!({"passed": true, "layer_height": cli["layer_height"]}),
    );
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    let process = &view["preset_selection"]["process"];
    assert_eq!(
        process["outcome"],
        if created {
            "project_local"
        } else {
            "existing_clean"
        }
    );
    for field in [
        "name",
        "system",
        "external",
        "project_local",
        "visible",
        "filament_id",
        "version",
    ] {
        assert_eq!(
            process["selected"][field], native["selected"][field],
            "{field}"
        );
    }
    assert_eq!(process["dirty"], false);
    assert_eq!(process["dirty_options"], json!([]));
    assert_eq!(view["settings"]["print_settings_id"], name);
}

// filament_extruder_override_keys (PrintConfig.cpp:97-122) and the ironing keys
// is_filament_extruder_override_key adds (PrintConfig.cpp:130-139).
const FILAMENT_OVERRIDE_KEYS: [&str; 24] = [
    "filament_retraction_length",
    "filament_z_hop",
    "filament_z_hop_types",
    "filament_retract_lift_above",
    "filament_retract_lift_below",
    "filament_retract_lift_enforce",
    "filament_retraction_speed",
    "filament_deretraction_speed",
    "filament_retract_restart_extra",
    "filament_retraction_minimum_travel",
    "filament_retract_length_toolchange",
    "filament_retract_restart_extra_toolchange",
    "filament_wipe_distance",
    "filament_retract_when_changing_layer",
    "filament_wipe",
    "filament_retract_before_wipe",
    "filament_retract_after_wipe",
    "filament_long_retractions_when_cut",
    "filament_retraction_distances_when_cut",
    "filament_retract_length_nc",
    "filament_ironing_flow",
    "filament_ironing_spacing",
    "filament_ironing_inset",
    "filament_ironing_speed",
];

// The plater completes the project config with FullPrintConfig::defaults() (Plater.cpp:9463)
// and builds the project-local filament preset from it (load_external_preset,
// PresetBundle.cpp:6354), so an override key the project lacks holds its option default, not
// the default filament preset's nil (PresetBundle.cpp:508-518).
#[tokio::test]
async fn project_local_filament_takes_the_option_defaults_the_project_lacks() {
    let Some(case) = Case::new(
        "selection-filament-option-defaults",
        input::settings("Missing Process", "0.2"),
    ) else {
        return;
    };
    let native = &case.native["projects"][0]["result"];
    // Native premise: the filament is project-local and the plater's config holds the default.
    assert_eq!(
        native["filament_presets"],
        json!(["Missing Filament(selection.3mf)"])
    );
    let plater: Value = serde_json::from_str(native["full_fff_config"].as_str().unwrap()).unwrap();
    assert_eq!(plater["filament_retract_length_nc"], json!(["10"]));
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    let actual =
        FILAMENT_OVERRIDE_KEYS.map(|key| (key, super::support::wire(key, &view["settings"][key])));
    case.write("api-override-keys.json", &json!(actual));
    for (key, value) in actual {
        assert_eq!(value, super::support::wire(key, &plater[key]), "{key}");
    }
}

// full_fff_config erases compatible_printers_condition (PresetBundle.cpp:5790-5794), so the
// plater's project config lacks it even where the 3MF stores it.
#[tokio::test]
async fn project_config_drops_the_stored_compatible_printers_condition() {
    let mut settings = input::settings("Missing Process", "0.2");
    settings["compatible_printers_condition"] = json!("");
    let Some(case) = Case::new("selection-erased-printers-condition", settings) else {
        return;
    };
    let native = &case.native["projects"][0]["result"];
    let plater: Value = serde_json::from_str(native["full_fff_config"].as_str().unwrap()).unwrap();
    // Native premise: the plater's config lacks the key the project stores.
    assert_eq!(plater.get("compatible_printers_condition"), None);
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    assert_eq!(view["settings"].get("compatible_printers_condition"), None);
}
