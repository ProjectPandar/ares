use super::*;

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
        let outcome = OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, 0, None);
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
        let outcome = OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, 0, None);
        assert_eq!(outcome.status, "PASS", "{key}: {}", outcome.detail);
        assert!(sweep::require_complete(&[outcome]).is_ok(), "{key}");
    }
}
