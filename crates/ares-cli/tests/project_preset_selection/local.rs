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
