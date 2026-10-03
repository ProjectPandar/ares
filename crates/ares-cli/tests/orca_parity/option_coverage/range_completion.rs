use super::*;

#[test]
fn floating_literal_domains_use_declared_endpoints_and_interior() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for (key, minimum, maximum) in [
        ("default_junction_deviation", 0.0, 0.3),
        ("machine_max_junction_deviation", 0.0, 0.3),
        ("print_flow_ratio", 0.01, 2.0),
        ("wall_maximum_resolution", 0.005, 0.5),
        ("wall_maximum_deviation", 0.005, 0.05),
        ("fuzzy_skin_point_distance", 0.01, 5.0),
        ("fuzzy_skin_scale", 0.1, 500.0),
        ("fuzzy_skin_persistence", 0.01, 1.0),
        ("tree_support_tip_diameter", 0.1, 100.0),
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        assert_eq!(plan.cases.len(), 3, "{key}");
        let values = plan
            .cases
            .iter()
            .map(|case| {
                let value = case.value.as_ref().unwrap();
                let scalar = value.as_array().map_or(value, |items| &items[0]);
                scalar.as_str().unwrap().parse::<f64>().unwrap()
            })
            .collect::<Vec<_>>();
        assert_eq!(&values[..2], &[minimum, maximum], "{key}");
        assert!(minimum < values[2] && values[2] < maximum, "{key}");
        assert!(plan.provably_legal, "{key}");
        assert!(plan.omission.is_none(), "{key}");
    }
}

#[test]
fn bounded_literal_seeds_use_the_schema_range_not_gui_warning_thresholds() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for key in ["spiral_mode_max_xy_smoothing", "support_threshold_overlap"] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let value = |label| {
            plan.cases
                .iter()
                .find(|case| case.label == label)
                .unwrap()
                .value
                .as_ref()
                .unwrap()
                .as_str()
                .unwrap()
                .trim_end_matches('%')
        };
        assert_eq!(value("seeded-literal"), value("seeded"), "{key}");
    }
}

#[test]
fn numeric_local_probes_cannot_complete_missing_endpoints() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for key in [
        "travel_speed",
        "outer_wall_speed",
        "filament_cost",
        "nozzle_diameter",
        "adaptive_bed_mesh_margin",
        "z_offset",
        "overhang_reverse_threshold",
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let outcome = OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None);
        assert_eq!(outcome.status, "INCOMPLETE", "{key}");
        assert_eq!(outcome.compared, plan.cases.len(), "{key}");
        assert!(
            plan.omission.is_some_and(|reason| !reason.is_empty()),
            "{key}"
        );
        assert!(sweep::require_complete(&[outcome]).is_err(), "{key}");
    }
}

#[test]
fn verified_numeric_domains_still_complete_after_all_cases_compare() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    for key in [
        "skirt_height",
        "hot_plate_temp_initial_layer",
        "temperature_vitrification",
        "chamber_temperature",
        "support_air_filtration",
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        let outcome = OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None);
        assert_eq!(outcome.status, "PASS", "{key}: {}", outcome.detail);
        assert!(sweep::require_complete(&[outcome]).is_ok(), "{key}");
    }
}
