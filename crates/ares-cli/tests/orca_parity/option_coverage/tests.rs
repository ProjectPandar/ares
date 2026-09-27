use super::{OptionOutcome, domains, inject_case};
use crate::runner;

#[test]
fn option_domain_plan_is_complete_and_source_cited() {
    let plans = domains::load(&runner::repo_root());

    assert_eq!(plans.len(), 650);
    assert!(plans.iter().all(|plan| {
        plan.source.starts_with("src/libslic3r/PrintConfig.cpp:")
            || plan.source.starts_with("src/libslic3r/PrintConfig.hpp:")
    }));
    assert!(plans.iter().all(|plan| !plan.cases.is_empty()));
    // Omissions are allowed for authored numeric probe domains (interior
    // samples of a schema range whose bounds upstream slice-time
    // validation rejects) and for schema-unbounded string/int/point
    // domains where only the baseline value is sliceable.
    assert!(
        plans.iter().all(|plan| {
            plan.omission.is_none()
                || plan
                    .omission
                    .is_some_and(|note| note.contains("authored") || note.contains("unbounded"))
        }),
        "unexpected omission: {:?}",
        plans
            .iter()
            .filter_map(|plan| plan.omission.map(|note| (plan.key.as_str(), note)))
            .collect::<Vec<_>>()
    );
}

#[test]
fn option_domain_plan_exhausts_explicit_bool_enum_and_bounded_range_values() {
    let plans = domains::load(&runner::repo_root());
    let booleans = plans
        .iter()
        .filter(|plan| matches!(plan.option_type.as_str(), "coBool" | "coBools"))
        .collect::<Vec<_>>();
    assert!(!booleans.is_empty());
    assert!(booleans.iter().all(|plan| plan.cases.len() == 2));

    let enums = plans
        .iter()
        .filter(|plan| matches!(plan.option_type.as_str(), "coEnum" | "coEnums"))
        .collect::<Vec<_>>();
    assert!(enums.len() >= 44);
    assert!(
        enums.iter().all(|plan| plan.cases.len() >= 2),
        "missing enum domains: {:?}",
        enums
            .iter()
            .filter(|plan| plan.cases.len() < 2)
            .map(|plan| plan.key.as_str())
            .collect::<Vec<_>>()
    );

    let ranges = plans
        .iter()
        .filter(|plan| plan.cases.iter().any(|case| case.label == "seeded"))
        .collect::<Vec<_>>();
    assert!(ranges.len() >= 100);
    // Bounded ranges emit min+max+seeded; `coFloatOrPercent` also emits a
    // `seeded-literal` absolute case; min-only-bounded domains (`[min,∞)`)
    // emit min+seeded — no `max` exists in the schema to probe.
    assert!(ranges.iter().all(|plan| {
        let labels = plan
            .cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>();
        labels == ["min", "max", "seeded"]
            || labels == ["min", "seeded"]
            || labels == ["min", "max", "seeded", "seeded-literal"]
            || labels == ["min", "seeded", "seeded-literal"]
    }));

    let baselines = plans
        .iter()
        .filter(|plan| plan.cases[0].label == "baseline")
        .collect::<Vec<_>>();
    assert!(
        baselines.iter().all(|plan| plan.cases[0].value.is_none()),
        "baseline cases carry no injected value"
    );
}

#[test]
fn enum_domains_use_only_active_definition_values() {
    let plans = domains::load(&runner::repo_root());
    let values = |key: &str| {
        plans
            .iter()
            .find(|plan| plan.key == key)
            .unwrap()
            .cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>()
    };

    assert_eq!(
        values("bottom_surface_pattern"),
        [
            "alignedrectilinear",
            "archimedeanchords",
            "concentric",
            "hilbertcurve",
            "monotonic",
            "monotonicline",
            "octagramspiral",
            "rectilinear",
        ]
    );
    assert_eq!(
        values("internal_solid_infill_pattern"),
        values("bottom_surface_pattern")
    );
    assert_eq!(
        values("gcode_flavor"),
        ["klipper", "marlin", "marlin2", "repetier", "reprapfirmware"]
    );
}

#[test]
fn generated_cases_are_injected_into_inventory_owner() {
    let plans = domains::load(&runner::repo_root());
    for (key, target_index) in [
        ("gcode_flavor", 0),
        ("detect_thin_wall", 1),
        ("enable_pressure_advance", 2),
        ("wipe", 0),
    ] {
        let plan = plans
            .iter()
            .find(|plan| plan.key == key && !plan.cases.is_empty())
            .unwrap();
        let mut machine = serde_json::Map::new();
        let mut process = serde_json::Map::new();
        let mut filaments = vec![serde_json::Map::new()];

        inject_case(
            plan,
            &plan.cases[0],
            &mut machine,
            &mut process,
            &mut filaments,
        );

        let targets = [&machine, &process, &filaments[0]];
        assert!(targets[target_index].contains_key(key), "{key}");
    }
}

#[test]
fn upstream_rejection_does_not_mask_compared_case_parity() {
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "bridge_flow")
        .unwrap();

    let passing = OptionOutcome::executed(&plan, 2, vec!["min: rejected".to_owned()], 0, None);
    let failing = OptionOutcome::executed(
        &plan,
        2,
        vec!["min: rejected".to_owned()],
        0,
        Some("max: divergent".to_owned()),
    );

    assert_eq!(passing.status, "INCOMPLETE");
    assert_eq!((passing.compared, passing.rejected), (2, 1));
    assert_eq!(failing.status, "FAIL");
}

#[test]
fn width_options_use_context_bounded_domain() {
    // `coFloatOrPercent` `*_line_width` options validate `max` against
    // percent and `max_literal` against mm (`PrintConfig.cpp`
    // `is_value_valid`); the reachable slicing bound is far below both
    // (`Print::validate` caps width by nozzle/flow — `10`/`10%`/`1000%`
    // all fail the oracle, `500%` passes). They must route through
    // `widths::generate`'s context-max model (mm `2.0` / percent `500`),
    // not the raw schema bound.
    let plans = domains::load(&runner::repo_root());
    for key in [
        "outer_wall_line_width",
        "inner_wall_line_width",
        "initial_layer_line_width",
        "sparse_infill_line_width",
        "top_surface_line_width",
        "internal_solid_infill_line_width",
        "support_line_width",
    ] {
        let plan = plans.iter().find(|plan| plan.key == key).unwrap();
        assert!(
            plan.width_domain.is_some(),
            "{key} must carry a context-bounded width_domain"
        );
        let labels: Vec<_> = plan.cases.iter().map(|case| case.label.as_str()).collect();
        assert!(
            labels.iter().all(|label| *label != "max"),
            "{key} must not emit a raw-schema `max` case: {labels:?}"
        );
        assert!(
            labels.contains(&"mm-context-max") && labels.contains(&"percent-context-max"),
            "{key} must emit context-max cases: {labels:?}"
        );
    }
}

#[test]
fn seeded_range_values_are_repeatable() {
    let first = domains::load(&runner::repo_root());
    let second = domains::load(&runner::repo_root());

    let values = |plans: &[domains::OptionPlan]| {
        plans
            .iter()
            .flat_map(|plan| &plan.cases)
            .filter(|case| case.label == "seeded")
            .map(|case| case.value.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(values(&first), values(&second));
}

#[test]
fn provably_legal_domains_cover_enumerable_and_bounded_options() {
    // Enumerable (bool/enum), schema-bounded (`min`+`max`), min-only-bounded
    // (`[min,∞)` covered by min+seeded), and context-bounded width domains
    // are all provably legal. Only options with no parseable schema bound at
    // all (strings, points, gcode blobs, unbounded numerics) stay
    // baseline-only — their domain has no min/max to exercise.
    let plans = domains::load(&runner::repo_root());
    let legal = plans.iter().filter(|plan| plan.provably_legal).count();
    let baseline = plans
        .iter()
        .filter(|plan| !plan.provably_legal)
        .map(|plan| plan.key.clone())
        .collect::<Vec<_>>();
    assert!(
        legal >= 519,
        "provably_legal regressed: {legal}; baseline-only: {baseline:?}"
    );
    // Every unbounded remainder is a type that cannot express min/max,
    // or a key with authored interior probes (schema bound rejected by
    // upstream slice-time validation — e.g. `skeleton_infill_line_width`
    // min trips "Too small line width", `PrintConfig.cpp:4064-4082`).
    for key in &baseline {
        let authored = plans
            .iter()
            .find(|plan| &plan.key == key)
            .and_then(|plan| plan.omission)
            .is_some_and(|note| note.contains("authored"));
        assert!(
            authored || !key.ends_with("_line_width"),
            "width option {key} must be provably legal via width_domain"
        );
    }
}
