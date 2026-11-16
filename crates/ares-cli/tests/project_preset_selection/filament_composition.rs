use serde_json::{Map, Value, json};

use super::{input, support::Case};

// Two filament presets of a two-extruder printer whose variants are Standard and High Flow.
// The project maps the TPU to the right (High Flow) extruder; its filament vectors are the
// collapsed one-per-filament form and lack keys the presets hold.
fn filament(name: &str, id: &str, temperature: [&str; 2], cost: &str, retract: [&str; 2]) -> Value {
    json!({
        "type": "filament", "name": name, "from": "system", "instantiation": "true",
        "version": "02.05.00.00", "filament_id": id,
        "filament_extruder_variant": ["Direct Drive Standard", "Direct Drive High Flow"],
        "nozzle_temperature": temperature, "filament_cost": [cost],
        "filament_retract_length_nc": retract, "filament_diameter": ["1.75"]
    })
}

fn dual_extruder_settings(filaments: &[&str]) -> Value {
    let count = filaments.len();
    let mut settings = input::settings("Missing Process", "0.2");
    settings["filament_settings_id"] = json!(filaments);
    settings["filament_ids"] = json!(["GFU99", "GFA00"][..count]);
    settings["filament_colour"] = json!(["#AABBCC", "#CCDDEE"][..count]);
    settings["filament_diameter"] = json!(["1.75", "1.75"][..count]);
    settings["nozzle_temperature"] = json!(["240", "240"][..count]);
    settings["filament_cost"] = json!(["20", "24.99"][..count]);
    settings["filament_extruder_variant"] =
        json!(["Direct Drive Standard", "Direct Drive Standard"][..count]);
    settings["filament_self_index"] = json!(["1", "2"][..count]);
    settings["filament_map"] = json!(["2", "1"][..count]);
    settings["extruder_variant_list"] = json!([
        "Direct Drive Standard,Direct Drive High Flow",
        "Direct Drive Standard,Direct Drive High Flow"
    ]);
    settings["nozzle_diameter"] = json!(["0.4", "0.4"]);
    settings["nozzle_volume_type"] = json!(["Standard", "High Flow"]);
    settings["different_settings_to_system"] = json!(vec![""; count + 2]);
    settings
}

fn presets() -> [Value; 2] {
    [
        filament(
            "Acme TPU @H2D",
            "acme-tpu",
            ["230", "235"],
            "20",
            ["14", "15"],
        ),
        filament(
            "Acme PLA @H2D",
            "acme-pla",
            ["220", "225"],
            "15.99",
            ["2", "3"],
        ),
    ]
}

#[tokio::test]
async fn always_composes_every_filament_from_its_selected_preset() {
    composition(
        "selection-filament-composition",
        &["Acme TPU @H2D", "Acme PLA @H2D"],
        presets(),
    )
    .await;
}

#[tokio::test]
async fn always_applies_the_single_selected_filament_preset() {
    composition("selection-filament-single", &["Acme TPU @H2D"], presets()).await;
}

// Like the H2D presets, each preset holds slow_down_min_speed once per extruder variant.
// Loading it keeps one value per filament (Preset::normalize, Preset.cpp:576-591, 1947), while
// a variant option such as filament_max_volumetric_speed keeps every variant.
#[tokio::test]
async fn always_keeps_one_value_per_filament_for_options_without_variants() {
    let mut presets = presets();
    for (preset, (slow_down, volumetric)) in presets
        .iter_mut()
        .zip([(["10", "12"], ["3.2", "3.6"]), (["20", "22"], ["25", "40"])])
    {
        preset["slow_down_min_speed"] = json!(slow_down);
        preset["filament_max_volumetric_speed"] = json!(volumetric);
    }
    composition(
        "selection-filament-normalized",
        &["Acme TPU @H2D", "Acme PLA @H2D"],
        presets,
    )
    .await;
}

async fn composition(label: &str, filaments: &[&str], presets: [Value; 2]) {
    let catalog = input::catalog(&presets, &[]);
    let Some(case) = Case::with_catalog(label, dual_extruder_settings(filaments), catalog) else {
        return;
    };
    let native = &case.native["projects"][0]["result"];
    // Native premise: the project's unprotected filament values are refreshed from the system
    // presets, which are selected unchanged.
    assert_eq!(native["filament_presets"], json!(filaments));
    let expected = full_fff_config_filaments(native);
    case.write("native-composition.json", &Value::Object(expected.clone()));
    let view = case
        .api()
        .await
        .expect("valid Always request must load the project");
    let settings = &view["settings"];
    let actual = expected
        .keys()
        .map(|key| (key.clone(), typed(key, &settings[key])))
        .collect::<Map<_, _>>();
    case.write("api-composition.json", &Value::Object(actual.clone()));
    for (key, value) in &expected {
        assert_eq!(&actual[key], value, "{key}");
    }
}

/// The filament options `PresetBundle::full_fff_config(false)` composes from the selected
/// filament presets (`PresetBundle.cpp:5621-5651` for one filament, `5652-5774` for several), as
/// the plater writes them into the project config (`full_config_secure`, `Plater.cpp:19779`):
/// each filament's preset is `find_preset(name, true)`, the edited copy for the selected preset;
/// a scalar option is the first filament's; a vector is the filaments' values appended in order;
/// `filament_self_index` numbers each filament's `filament_extruder_variant` entries;
/// `filament_ids` are the presets' `filament_id` (`5636`, `5679`, `5821`).
fn full_fff_config_filaments(native: &Value) -> Map<String, Value> {
    let collection = &native["filament"];
    let presets = native["filament_presets"]
        .as_array()
        .unwrap()
        .iter()
        .map(|name| {
            if name == &collection["selected"]["name"] {
                &collection["edited"]
            } else {
                collection["records"]
                    .as_array()
                    .unwrap()
                    .iter()
                    .find(|record| &record["name"] == name)
                    .unwrap()
            }
        })
        .collect::<Vec<_>>();
    let mut composed = Map::new();
    for (key, first) in presets[0]["config"].as_object().unwrap() {
        // full_fff_config skips these (5727-5728), erases them (5791-5797), or sets them (5763-5773,
        // 5819).
        if matches!(
            key.as_str(),
            "compatible_prints"
                | "compatible_printers"
                | "compatible_prints_condition"
                | "compatible_printers_condition"
                | "inherits"
                | "different_settings_to_system"
                | "filament_settings_id"
                | "filament_self_index"
        ) {
            continue;
        }
        let value = if first["type"].as_u64().unwrap() & 0x4000 == 0 {
            decoded(key, first["serialized"].as_str().unwrap())
        } else {
            Value::Array(
                presets
                    .iter()
                    .flat_map(|preset| vector(key, preset))
                    .collect(),
            )
        };
        composed.insert(key.clone(), value);
    }
    let self_index = presets
        .iter()
        .enumerate()
        .flat_map(|(index, preset)| {
            vec![json!(index + 1); vector("filament_extruder_variant", preset).len()]
        })
        .collect();
    composed.insert("filament_self_index".to_owned(), Value::Array(self_index));
    composed.insert(
        "filament_ids".to_owned(),
        presets
            .iter()
            .map(|preset| preset["filament_id"].clone())
            .collect(),
    );
    composed
}

fn vector(key: &str, preset: &Value) -> Vec<Value> {
    match decoded(key, preset["config"][key]["serialized"].as_str().unwrap()) {
        Value::Array(values) => values,
        value => unreachable!("{key} is a vector option: {value}"),
    }
}

fn decoded(key: &str, serialized: &str) -> Value {
    serde_json::from_str(
        &ares_core::decode_config_value_by_key(key, serialized)
            .unwrap_or_else(|error| panic!("{key}: {error}")),
    )
    .unwrap()
}

/// An Always output value in the typed form `decoded` gives Native's.
fn typed(key: &str, value: &Value) -> Value {
    match value {
        Value::Null => Value::Null,
        value => decoded(key, &super::support::wire(key, value)),
    }
}
