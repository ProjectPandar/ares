//! PrintConfig.cpp:779-787; Config.cpp:321-337; BuildVolume.cpp:377-421;
//! Print.cpp:833-930,1354-1396: bounded scalar and generated-layer consumers.

use super::super::super::{OptionOutcome, domains};
use crate::runner;
use serde_json::json;

#[test]
fn orca_parity_printable_height_bounded_metadata_and_inventory() {
    let Some(plans) = domains::try_load(&runner::repo_root()) else {
        return;
    };
    let records = plans.iter().map(|plan| json!({
        "key":plan.key,"type":plan.option_type,"scope":plan.raw_scope,"source":plan.source,
        "cases":plan.cases.iter().map(|case|json!({"label":case.label,"value":case.value})).collect::<Vec<_>>(),
        "omission":plan.omission,"provably_legal":plan.provably_legal,"tool_metadata":plan.tool_metadata,
        "width_domain":plan.width_domain.as_ref().map(|domain|json!({"metadata":domain.metadata,"probes":domain.probes}))
    })).collect::<Vec<_>>();
    if std::env::var_os("ARES_PARITY_ARTIFACT_ROOT").is_some() {
        crate::artifacts::overwrite(
            &crate::artifacts::root_from_env()
                .unwrap()
                .join("printable-height-all-plans.json"),
            &serde_json::to_vec_pretty(&records).unwrap(),
        )
        .unwrap();
    }
    assert_eq!(plans.len(), 650);
    let plan = plans
        .iter()
        .find(|plan| plan.key == "printable_height")
        .unwrap();
    let mut unproved = plan.clone();
    unproved.provably_legal = false;
    unproved.omission = Some(
        "matching samples without scene, generated-layer and sequential witnesses remain unproved",
    );
    let outcome = OptionOutcome::executed(&unproved, unproved.cases.len(), Vec::new(), 0, None);
    assert_eq!(outcome.status, "INCOMPLETE");
    assert!(!outcome.detail.is_empty());
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.clone().unwrap())
            .collect::<Vec<_>>(),
        [json!("50"), json!("400"), json!("214700")]
    );
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
}
