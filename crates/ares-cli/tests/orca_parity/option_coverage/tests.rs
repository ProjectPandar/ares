use super::{OptionOutcome, domains, inject_case};
use crate::runner;

#[test]
fn option_domain_plan_is_complete_and_source_cited() {
    let plans = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    };

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
    let plans = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    };
    let booleans = plans
        .iter()
        .filter(|plan| matches!(plan.option_type.as_str(), "coBool" | "coBools"))
        .collect::<Vec<_>>();
    assert!(!booleans.is_empty());
    // Non-nullable booleans cover {0,1}; nullable ones add the `nil`
    // probe (the key is then omitted from the exported config).
    assert!(booleans.iter().all(|plan| plan.cases.len() >= 2));

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
            || labels
                == [
                    "min",
                    "max",
                    "seeded",
                    "min-literal",
                    "max-literal",
                    "seeded-literal",
                ]
            || labels
                == ["min", "seeded", "min-literal", "max-literal", "seeded-literal"]
            || labels == ["min", "seeded", "min-literal", "seeded-literal"]
            // Upper-only bounds (`nozzle_diameter` max=100) probe the
            // declared ceiling and a seeded interior below it.
            || labels == ["max", "seeded"]
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
    let plans = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    };
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
    let plans = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    };
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
    let plan = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    }
    .into_iter()
    .find(|plan| plan.key == "bridge_flow")
    .unwrap();

    let passing = OptionOutcome::executed(&plan, 2, vec!["min: rejected".to_owned()], 0, 0, None);
    let failing = OptionOutcome::executed(
        &plan,
        2,
        vec!["min: rejected".to_owned()],
        0,
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
    let plans = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    };
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
    let first = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    };
    let second = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    };

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
    let plans = match domains::try_load(&runner::repo_root()) {
        Some(v) => v,
        None => return,
    };
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

/// The fixture inventory is a curated subset of upstream's
/// `this->add("key", ...)` set — CLI action flags (`slice`,
/// `export_gcode`), SLA options, placeholder variables, and
/// GUI/networking metadata never reach slicing. This test reconciles
/// the fixture against the upstream key set: every upstream key absent
/// from the inventory must be listed in
/// `tests/parity/upstream-key-exclusions.txt` — so a key can never
/// silently drop out of coverage tracking without a deliberate record.
#[test]
fn option_inventory_reconciles_with_upstream_key_set() {
    let repo = runner::repo_root();
    let Some(covered_plans) = domains::try_load(&repo) else {
        return;
    };
    let source =
        std::fs::read_to_string(repo.join("OrcaSlicer/src/libslic3r/PrintConfig.cpp")).unwrap();
    let upstream: std::collections::BTreeSet<String> =
        regex::Regex::new(r#"this->add\("([a-zA-Z0-9_]+)""#)
            .unwrap()
            .captures_iter(&source)
            .map(|captures| captures[1].to_owned())
            .collect();
    let covered: std::collections::BTreeSet<String> =
        covered_plans.iter().map(|plan| plan.key.clone()).collect();
    let exclusions: std::collections::BTreeSet<String> =
        std::fs::read_to_string(repo.join("tests/parity/upstream-key-exclusions.txt"))
            .unwrap()
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && !line.starts_with('#'))
            .map(ToOwned::to_owned)
            .collect();
    let uncovered: std::collections::BTreeSet<String> =
        upstream.difference(&covered).cloned().collect();
    assert_eq!(
        uncovered, exclusions,
        "upstream keys missing from the option inventory must match the \
         recorded exclusion list (add coverage or record the exclusion)"
    );
}
