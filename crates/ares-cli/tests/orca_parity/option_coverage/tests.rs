use super::domains;
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
        .filter(|plan| plan.omission.is_none())
        .collect::<Vec<_>>();
    assert!(enums.len() >= 30);
    assert!(enums.iter().all(|plan| plan.cases.len() >= 2));

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
