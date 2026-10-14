use super::{bounds, numeric_domains};
use serde_json::json;

#[test]
fn generated_axis_option_resolves_cpp_definition_instead_of_header_line_number() {
    let lines = [
        "def = this->add(\"unrelated\", coFloat);",
        "def->min = 71;",
        "def->max = 72;",
        "def = this->add(\"machine_max_acceleration_\" + axis.name, coFloats);",
        "def->min = 0;",
        "def = this->add(\"machine_max_jerk_\" + axis.name, coFloats);",
        "def->min = 0;",
        "def = this->add(\"next\", coFloat);",
    ];
    for axis in ["x", "y", "z", "e"] {
        let block =
            bounds::definition_block(&lines, &format!("machine_max_acceleration_{axis}"), 2);
        assert_eq!(bounds::lower_bound(&block), Some(0.0));
        assert_eq!(bounds::upper_bound(&block), None);
        assert!(block.starts_with("def = this->add(\"machine_max_acceleration_\""));
    }
}

#[test]
fn implicit_integer_storage_limits_are_not_generated_as_source_boundaries() {
    let entry = json!({"wire_shape":"scalar"});
    for key in [
        "slow_down_layers",
        "bottom_shell_layers",
        "top_shell_layers",
        "elefant_foot_compensation_layers",
    ] {
        let (cases, omission, complete) =
            numeric_domains::generate(&entry, key, "coInt", "def->min = 0;");
        assert!(complete, "{key}");
        assert_eq!(omission, None);
        for case in cases {
            let value: i32 = case.value.unwrap().as_str().unwrap().parse().unwrap();
            assert!((0..=5).contains(&value), "{key}: {value}");
            assert_ne!(case.label, "max");
        }
    }
}

#[test]
fn source_named_integer_samples_and_resolution_complete_only_the_declared_scope() {
    let Some(plans) = super::try_load(&crate::runner::repo_root()) else {
        return;
    };
    for key in [
        "resolution",
        "enable_long_retraction_when_cut",
        "min_bead_width",
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        assert!(plan.provably_legal, "{key}: {:?}", plan.omission);
        assert!(
            plan.cases
                .iter()
                .all(|case| !matches!(case.label.as_str(), "max" | "max-literal"))
        );
    }
}

#[test]
fn context_and_baseline_domains_are_not_promoted_by_the_unbounded_rule() {
    let Some(plans) = super::try_load(&crate::runner::repo_root()) else {
        return;
    };
    let identity = plans
        .iter()
        .find(|plan| plan.key == "filament_self_index")
        .unwrap();
    assert!(
        identity.provably_legal,
        "filament_self_index: bounded Native slot witnesses"
    );
    assert!(identity.omission.is_none(), "filament_self_index");
    assert_eq!(
        identity
            .cases
            .iter()
            .map(|case| case.value.clone())
            .collect::<Vec<_>>(),
        [
            Some(json!(["1", "2"])),
            Some(json!(["2", "1"])),
            Some(json!(["1", "1", "2", "2"])),
            Some(json!(["2", "2", "1", "1"]))
        ]
    );
    assert_eq!(
        identity.tool_metadata.as_ref().unwrap()["domain"],
        "bounded H2D filament self-index slots"
    );

    let mapping = plans
        .iter()
        .find(|plan| plan.key == "filament_map")
        .unwrap();
    assert!(
        mapping.provably_legal,
        "filament_map: bounded G4 Manual scope"
    );
    assert!(mapping.omission.is_none(), "filament_map");
    assert_eq!(
        mapping.tool_metadata,
        Some(json!({
            "domain":"G4 bounded Manual filament maps",
            "definition":"PrintConfig.cpp:2489",
            "consumer":"ToolOrdering.cpp:1275-1312",
            "input":"tool_context::mapping",
            "mode":"Manual",
            "nozzle_diameter":["0.4","0.4"],
            "nozzle_volume_type":["Standard","Standard"],
            "wall_filament":1,"sparse_infill_filament":2,
            "deferred":["automatic grouping","dynamic mapping","all vector shapes"]
        }))
    );
    assert_eq!(
        mapping
            .cases
            .iter()
            .map(|case| case.value.clone().unwrap())
            .collect::<Vec<_>>(),
        [
            json!(["1", "1"]),
            json!(["1", "2"]),
            json!(["2", "1"]),
            json!(["2", "2"])
        ]
    );
    assert_eq!(
        mapping
            .cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>(),
        ["manual-1-1", "manual-1-2", "manual-2-1", "manual-2-2"]
    );
    for key in ["layer_height", "initial_layer_print_height"] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        assert!(plan.provably_legal, "{key}: bounded one-nozzle context");
        assert!(plan.omission.is_none(), "{key}");
    }
    for key in ["bridge_flow", "internal_bridge_flow"] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        assert!(plan.provably_legal, "{key}: bounded activated bridge scope");
        assert!(plan.omission.is_none(), "{key}");
        assert_eq!(
            plan.cases
                .iter()
                .map(|case| case.value.as_ref().unwrap().as_str().unwrap())
                .collect::<Vec<_>>(),
            ["0.5", "1", "1.5", "2"]
        );
    }
    for (key, wires) in [
        ("filament_flow_ratio", ["0.5", "1", "1.5", "2"]),
        ("filament_diameter", ["1", "1.75", "2.85", "3"]),
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        assert!(plan.provably_legal, "{key}: bounded single-filament scope");
        assert!(plan.omission.is_none(), "{key}");
        assert_eq!(
            plan.cases
                .iter()
                .map(|case| case.value.clone().unwrap())
                .collect::<Vec<_>>(),
            wires.map(|wire| json!([wire]))
        );
    }
    for key in ["skin_infill_line_width", "skeleton_infill_line_width"] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        assert!(plan.provably_legal, "{key}: bounded LockedZag context");
        assert!(plan.omission.is_none(), "{key}");
        assert!(plan.width_domain.is_some(), "{key}");
    }
    let grouped = plans
        .iter()
        .find(|plan| plan.key == "extruder_printable_area")
        .unwrap();
    assert!(
        grouped.provably_legal,
        "extruder_printable_area bounded authored scope"
    );
    assert!(
        grouped.omission.is_none(),
        "extruder_printable_area is not numeric"
    );
}

#[test]
fn declared_upper_only_scope_is_complete_without_an_invented_lower_endpoint() {
    let entry = json!({"wire_shape": "array"});
    let (cases, omission, complete) =
        numeric_domains::generate(&entry, "nozzle_diameter", "coFloats", "def->max = 100;");
    assert_eq!(
        cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>(),
        ["max", "seeded"]
    );
    assert!(complete);
    assert_eq!(omission, None);
}

#[test]
fn declared_minimum_and_existing_interior_complete_an_unbounded_upper_scope() {
    let entry = json!({"wire_shape": "scalar"});
    for (key, kind) in [
        ("bridge_speed", "coFloat"),
        ("resolution", "coFloat"),
        ("bottom_surface_filament_id", "coInt"),
        ("tree_support_top_rate", "coPercent"),
    ] {
        let (cases, omission, complete) =
            numeric_domains::generate(&entry, key, kind, "def->min = 0;");
        let expected: &[&str] = if key == "tree_support_top_rate" {
            &["min", "seeded", "probe"]
        } else {
            &["min", "seeded"]
        };
        assert_eq!(
            cases
                .iter()
                .map(|case| case.label.as_str())
                .collect::<Vec<_>>(),
            expected
        );
        assert!(complete, "{key}");
        assert_eq!(omission, None, "{key}");
        assert_eq!(
            cases[0].value,
            Some(json!(if kind == "coPercent" { "0%" } else { "0" }))
        );
        let interior: f64 = cases[1]
            .value
            .as_ref()
            .unwrap()
            .as_str()
            .unwrap()
            .trim_end_matches('%')
            .parse()
            .unwrap();
        assert!((0.0..5.0).contains(&interior), "{key}: {interior}");
    }
}

#[test]
fn source_unbounded_floats_keep_local_samples_without_claiming_extrema() {
    let entry = json!({"wire_shape": "array"});
    let (cases, omission, complete) =
        numeric_domains::generate(&entry, "retraction_length", "coFloats", "");
    assert_eq!(
        cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>(),
        ["probe-min", "probe-seeded"]
    );
    assert!(complete);
    assert_eq!(omission, None);
    assert_eq!(cases[0].value, Some(json!(["0.5"])));
}

#[test]
fn unknown_expression_is_not_treated_as_an_unbounded_source_side() {
    let entry = json!({"wire_shape": "scalar"});
    let (cases, omission, complete) = numeric_domains::generate(
        &entry,
        "unknown",
        "coFloat",
        "def->min = 0; def->max = scale * 10;",
    );
    assert!(!complete);
    assert!(omission.unwrap().contains("unsupported"));
    assert!(cases.iter().all(|case| case.label != "max"));
}

#[test]
fn min_bead_width_definition_stops_before_dynamic_nullable_metadata_copy() {
    let source = [
        "def = this->add(\"min_bead_width\", coPercent);",
        "def->min = 0;",
        "for (auto& opt_key : filament_extruder_override_keys) {",
        "def = this->add_nullable(opt_key, it_opt->second.type);",
        "def->min = it_opt->second.min;",
        "def->max = it_opt->second.max;",
        "}",
        "def = this->add(\"next\", coBool);",
    ];
    let block = bounds::definition_block(&source, "min_bead_width", 1);
    assert_eq!(bounds::lower_bound(&block), Some(0.0));
    assert_eq!(bounds::upper_bound(&block), None);
    assert!(!block.contains("it_opt->second.max"));
    let (cases, omission, complete) = numeric_domains::generate(
        &json!({"wire_shape": "scalar"}),
        "min_bead_width",
        "coPercent",
        &block,
    );
    assert!(complete);
    assert_eq!(omission, None);
    assert_eq!(cases[0].value, Some(json!("0%")));
    assert!(cases.iter().all(|case| case.label != "max"));
}

#[test]
fn generated_nullable_option_uses_declared_base_bounds_not_neighbor_metadata() {
    let source = [
        "def = this->add(\"retraction_distances_when_cut\", coFloats);",
        "def->min = 10;",
        "def->max = 18;",
        "def = this->add(\"unrelated\", coInt);",
        "def->min = 100;",
        "def->max = 200;",
    ];
    let block = bounds::definition_block(&source, "filament_retraction_distances_when_cut", 5);
    assert_eq!(bounds::numeric_bounds(&block), Some((10.0, 18.0)));
}
