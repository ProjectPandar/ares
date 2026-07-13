use std::collections::BTreeMap;

use serde_json::Value;

use super::super::process_object_source::{expected::DECLARATION_ORDER, fixture_fields};
use super::super::super::{
    FloatOrPercent, OrcaBool, OrcaFloat, OrcaInt, ProcessObjectSourceOptions,
    ProcessPerimeterGenerator, ProcessSeamPosition, ProcessSupportStyle, ProcessSupportType,
    ProjectPrintSourceOptions,
};
use super::{ObjectOptionOverrides, ObjectOptions, types};

const FIXTURE: &[u8] =
    include_bytes!("../../../../../../tests/ksr_fdmtest_v4/ksr_fdmtest_v4.project.3mf");

const FIXTURE_DIFFERENCES: [(&str, &str, &str); 18] = [
    ("brim_object_gap", "0", "0.1"),
    ("brim_width", "0", "5"),
    ("default_acceleration", "500", "10000"),
    ("elefant_foot_compensation", "0", "0.15"),
    ("initial_layer_acceleration", "300", "500"),
    ("inner_wall_acceleration", "10000", "0"),
    ("line_width", "0", "0.42"),
    ("max_bridge_length", "10", "0"),
    ("outer_wall_acceleration", "500", "5000"),
    ("support_interface_bottom_layers", "0", "2"),
    ("support_interface_top_layers", "3", "2"),
    ("support_line_width", "0", "0.42"),
    ("support_speed", "80", "150"),
    ("support_type", "normal(auto)", "tree(auto)"),
    ("top_surface_acceleration", "500", "2000"),
    ("tree_support_branch_angle", "40", "45"),
    ("tree_support_branch_diameter", "5", "2"),
    ("wall_generator", "arachne", "classic"),
];

#[test]
fn object_options_fixture_preserves_document_metadata_and_part_matrix() {
    let project = crate::load_project(FIXTURE).unwrap();
    let domain_object = project
        .objects()
        .iter()
        .find(|value| value.id() == 2)
        .unwrap();
    let settings = project
        .documents()
        .model_settings
        .objects
        .iter()
        .find(|value| value.id == domain_object.id())
        .unwrap();

    assert_eq!(settings.name, "ksr_fdmtest_v4.drc");
    assert!(settings.module.is_empty());
    assert_eq!(settings.overrides, ObjectOptionOverrides::default());
    assert_eq!(settings.retained_config.len(), 1);
    assert_eq!(settings.retained_config[0].key, "extruder");
    assert_eq!(settings.retained_config[0].value, "1");
    assert!(!settings.retained_config.iter().any(|entry| entry.key == "matrix"));
    assert!(!ObjectOptions::DECLARATION_ORDER.contains(&"matrix"));

    assert_eq!(settings.parts.len(), 1);
    let part = settings.parts.iter().find(|value| value.id == 1).unwrap();
    assert_eq!(part.subtype, "normal_part");
    assert_eq!(
        part.metadata
            .iter()
            .find(|entry| entry.key == "matrix")
            .unwrap()
            .value,
        "1 0 0 0 0 1 0 0 0 0 1 0 0 0 0 1"
    );
}

#[test]
fn object_options_fixture_resolves_typed_base_and_exact_difference_ledger() {
    let project = crate::load_project(FIXTURE).unwrap();
    let domain_object = project
        .objects()
        .iter()
        .find(|value| value.id() == 2)
        .unwrap();
    let settings = project
        .documents()
        .model_settings
        .objects
        .iter()
        .find(|value| value.id == domain_object.id())
        .unwrap();

    let base: ProcessObjectSourceOptions = serde_json::from_value(Value::Object(
        fixture_fields(DECLARATION_ORDER.iter().copied()),
    ))
    .unwrap();
    let print: ProjectPrintSourceOptions =
        serde_json::from_value(Value::Object(fixture_fields(["nozzle_diameter"]))).unwrap();
    let num_extruders = print.nozzle_diameter.0.len();
    assert_eq!(num_extruders, 2);
    assert_eq!(
        print.nozzle_diameter.0,
        [OrcaFloat(0.4), OrcaFloat(0.4)]
    );

    let effective = ObjectOptions::resolve(&base, &settings.overrides, num_extruders);
    assert_eq!(effective, ObjectOptions::from_base(&base));
    types::assert_base_and_sparse(&effective, &base, &settings.overrides);

    let fixture = serde_json::to_value(&base).unwrap();
    let fixture = fixture.as_object().unwrap();
    let defaults = serde_json::to_value(ProcessObjectSourceOptions::default()).unwrap();
    let defaults = defaults.as_object().unwrap();
    let differences = DECLARATION_ORDER
        .iter()
        .filter_map(|key| {
            let default = defaults[*key].as_str().unwrap();
            let value = fixture[*key].as_str().unwrap();
            (default != value).then_some((*key, (default, value)))
        })
        .collect::<BTreeMap<_, _>>();
    let expected = FIXTURE_DIFFERENCES
        .into_iter()
        .map(|(key, default, fixture)| (key, (default, fixture)))
        .collect::<BTreeMap<_, _>>();

    assert_eq!(differences, expected);
    assert_eq!(differences.len(), 18);
    assert_eq!(
        DECLARATION_ORDER
            .iter()
            .filter(|key| defaults[**key] == fixture[**key])
            .count(),
        108
    );

    assert_eq!(effective.layer_height, OrcaFloat(0.2));
    assert_eq!(effective.interface_shells, OrcaBool(false));
    assert_eq!(
        effective.line_width,
        FloatOrPercent::Float(0.42)
    );
    assert_eq!(
        effective.wall_generator,
        ProcessPerimeterGenerator::Classic
    );
    assert_eq!(effective.enable_support, OrcaBool(false));
    assert_eq!(effective.support_type, ProcessSupportType::TreeAuto);
    assert_eq!(effective.support_style, ProcessSupportStyle::Default);
    assert_eq!(effective.support_interface_top_layers, OrcaInt(2));
    assert_eq!(effective.support_filament, OrcaInt(0));
    assert_eq!(effective.support_interface_filament, OrcaInt(0));
    assert_eq!(effective.seam_position, ProcessSeamPosition::Aligned);
}
