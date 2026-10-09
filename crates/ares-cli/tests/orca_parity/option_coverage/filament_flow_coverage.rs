//! GCode.cpp:6467-6513 and Extruder.hpp:61: ratio and area-to-length conversion.
use super::*;
use serde_json::json;

#[path = "filament_flow_coverage/input.rs"]
pub(super) mod input;

#[path = "filament_flow_coverage/rejections.rs"]
mod rejections;

#[test]
fn orca_parity_filament_flow_ratio_bounded_output() {
    compare("filament_flow_ratio", &["0.5", "1", "1.5", "2"]);
}

#[test]
fn orca_parity_filament_diameter_bounded_output() {
    compare("filament_diameter", &["1", "1.75", "2.85", "3"]);
}

fn compare(key: &str, wires: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == key)
        .unwrap();
    let root = parity::artifacts::root_from_env().unwrap();
    let evidence = root.join(format!("{key}-bounded.json"));
    let mut proofs = Vec::new();
    let mut amounts = Vec::new();
    for wire in wires {
        let exported = input::export(
            &runner,
            key,
            wire,
            &format!("bounded-filament/{key}/{wire}"),
        )
        .unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let amount = role_extrusion(
            std::str::from_utf8(&sliced.reference).unwrap(),
            "Outer wall",
        );
        assert!(
            amount > 0.0,
            "{key}={wire}: missing ordinary wall extrusion; {root:?}"
        );
        let outcome = parity::compare_case(&sliced);
        proofs.push(
            json!({"requested":[wire], "role":"Outer wall", "positive_role_e":amount,
            "full_output_status":outcome.status, "detail":outcome.detail}),
        );
        parity::artifacts::overwrite(&evidence, &serde_json::to_vec_pretty(&proofs).unwrap())
            .unwrap();
        assert_eq!(
            outcome.status, "PASS",
            "{key}={wire}: {}; {root:?}",
            outcome.detail
        );
        amounts.push(amount);
    }
    assert!(
        amounts.windows(2).any(|pair| pair[0] != pair[1]),
        "filament consumption must change ordinary extrusion: {amounts:?}"
    );
    assert!(
        plan.cases.iter().all(|case| {
            let value = case.value.as_ref().unwrap()[0]
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap();
            wires
                .iter()
                .any(|wire| wire.parse::<f64>().unwrap() == value)
        }),
        "every declared request must have a full-output witness"
    );
    let outcome = OptionOutcome::executed(&plan, plan.cases.len(), Vec::new(), 0, None);
    parity::artifacts::overwrite(
        &evidence,
        &serde_json::to_vec_pretty(&json!({"key":key,"domain":input::metadata(),
            "cases":proofs,"classification":outcome.status,"detail":outcome.detail}))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(
        outcome.status, "PASS",
        "{key}: {}; {root:?}",
        outcome.detail
    );
}

fn role_extrusion(gcode: &str, role: &str) -> f64 {
    let mut selected = false;
    let mut relative = false;
    let mut amount = 0.0;
    for line in gcode.lines() {
        let command = line.split_whitespace().next();
        if command == Some("M83") {
            relative = true;
        } else if command == Some("M82") {
            relative = false;
        } else if let Some(value) = line.strip_prefix(";TYPE:") {
            selected = value == role;
        } else if selected && line.starts_with("G1 ") {
            let words = line.split_whitespace().collect::<Vec<_>>();
            let e = words
                .iter()
                .find_map(|word| word.strip_prefix('E'))
                .map(|word| word.parse::<f64>().unwrap());
            if let Some(e) = e
                && e > 0.0
                && words.iter().any(|word| word.starts_with(['X', 'Y']))
            {
                assert!(relative, "role observation requires verified relative E");
                amount += e;
            }
        }
    }
    amount
}
