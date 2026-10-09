//! Native-backed mapping admission before baseline-only classification changes.
use super::*;
use serde_json::json;
use std::collections::BTreeSet;

#[path = "tool_observation.rs"]
mod observation;
use observation::{motion_lines, observe};

#[test]
fn orca_parity_g4_manual_filament_maps_consume_both_nozzles_and_cover_finite_scope() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "filament_map")
        .unwrap();
    let requests = [
        json!(["1", "1"]),
        json!(["1", "2"]),
        json!(["2", "1"]),
        json!(["2", "2"]),
    ];
    let mut proofs = Vec::new();
    for requested in &requests {
        let map = requested
            .as_array()
            .unwrap()
            .iter()
            .map(|value| value.as_str().unwrap().parse::<usize>().unwrap())
            .collect::<Vec<_>>();
        let label = format!("g4-mapping/manual-{}-{}", map[0], map[1]);
        let exported = tool_context::mapping(&runner, &label, requested).unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let native = std::str::from_utf8(&sliced.reference).unwrap();
        let observations = observe(native);
        let outcome = parity::compare_case(&sliced);
        proofs.push(json!({
            "requested":requested, "label":label, "observations":observations,
            "full_output_status":outcome.status, "detail":outcome.detail
        }));
        write_proofs(&proofs);
        assert_eq!(outcome.status, "PASS", "{label}: {}", outcome.detail);
        assert!(
            motion_lines(native).any(|line| line == format!("M620.10 R{}", map[0])),
            "{label}: mapped slot0 retract not consumed"
        );
        assert!(
            motion_lines(native).any(|line| line == format!("M620.10 R{}", map[1])),
            "{label}: mapped slot1 retract not consumed"
        );
        let mut used = BTreeSet::new();
        let mut layers = BTreeSet::new();
        for (layer, filament, nozzle, role) in &observations {
            assert_eq!(*nozzle, map[*filament], "{label}: wrong mapped nozzle");
            let expected = if role == "Sparse infill" { 1 } else { 0 };
            assert_eq!(*filament, expected, "{label}: wrong wall/infill filament");
            used.insert(*filament);
            layers.insert(*layer);
        }
        assert_eq!(
            used,
            BTreeSet::from([0, 1]),
            "{label}: configured is not actually used"
        );
        assert!(layers.len() > 1, "{label}: missing multilayer usage");
    }
    let outcome = OptionOutcome::executed(&plan, proofs.len(), Vec::new(), 0, None);
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join("g4-mapping-bounded.json"),
        &serde_json::to_vec_pretty(&json!({
            "cases":proofs,"classification":outcome.status,"detail":outcome.detail,
            "scope":"four coherent Manual maps, two 0.4mm nozzles, wall1/infill2",
            "deferred":["automatic grouping","dynamic mapping","all vector shapes"]
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(outcome.status, "PASS", "filament_map: {}", outcome.detail);
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.as_ref().unwrap())
            .collect::<Vec<_>>(),
        requests.iter().collect::<Vec<_>>()
    );
}

fn write_proofs(proofs: &[Value]) {
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join("g4-mapping-cases.json"),
        &serde_json::to_vec_pretty(proofs).unwrap(),
    )
    .unwrap();
}
