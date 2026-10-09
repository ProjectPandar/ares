//! Native-backed mapping admission before baseline-only classification changes.
use super::*;
use serde_json::json;
use std::collections::BTreeSet;

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
        let exported = tool_context::mapping(&runner, &label, requested);
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
        let motion = native.split("; CONFIG_BLOCK_START").next().unwrap();
        assert!(
            motion.contains(&format!("M620.10 R{}", map[0])),
            "{label}: mapped slot0 retract not consumed"
        );
        assert!(
            motion.contains(&format!("M620.10 R{}", map[1])),
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

// Observe only genuine wall/sparse XY extrusion, excluding startup/flush moves
// and CONFIG_BLOCK template literals. Full comparison above is never filtered.
fn observe(native: &str) -> BTreeSet<(usize, usize, usize, String)> {
    let mut layer = 0;
    let mut active = None;
    let mut role = "";
    let mut observations = BTreeSet::new();
    for line in native.split("; CONFIG_BLOCK_START").next().unwrap().lines() {
        if line == "; CHANGE_LAYER" {
            layer += 1;
        }
        if let Some(marker) = line.strip_prefix("; G4_ACTIVE F") {
            let (filament, nozzle) = marker.split_once(" N").unwrap();
            active = Some((
                filament.parse::<usize>().unwrap(),
                nozzle.parse::<usize>().unwrap(),
            ));
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        if line.starts_with("G1 ") && matches!(role, "Outer wall" | "Inner wall" | "Sparse infill")
        {
            let fields = line.split_ascii_whitespace().collect::<Vec<_>>();
            let xy = fields
                .iter()
                .any(|field| field.starts_with('X') || field.starts_with('Y'));
            let extrusion = fields
                .iter()
                .filter_map(|field| field.strip_prefix('E'))
                .any(|value| value.parse::<f64>().unwrap() > 0.0);
            if xy && extrusion {
                let (filament, nozzle) = active.unwrap();
                observations.insert((layer, filament, nozzle, role.to_owned()));
            }
        }
    }
    observations
}
