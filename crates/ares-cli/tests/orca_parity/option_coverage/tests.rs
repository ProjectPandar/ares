use super::{domains, inject_case};
use crate::runner;

#[test]
fn option_domain_plan_is_complete_and_source_cited() {
    let plans = domains::load(&runner::repo_root());

    assert_eq!(plans.len(), 650);
    assert!(plans.iter().all(|plan| {
        plan.source.starts_with("src/libslic3r/PrintConfig.cpp:")
            || plan.source.starts_with("src/libslic3r/PrintConfig.hpp:")
    }));
    assert!(
        plans
            .iter()
            .all(|plan| plan.omission.is_some() ^ !plan.cases.is_empty())
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
    assert!(ranges.iter().all(|plan| {
        plan.cases
            .iter()
            .map(|case| case.label.as_str())
            .collect::<Vec<_>>()
            == ["min", "max", "seeded"]
    }));
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
