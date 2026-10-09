//! LayerRegion.cpp:31-58 and GCode.cpp:6478: bridge geometry and role flow.
use super::*;
use serde_json::json;

#[path = "bridge_flow_coverage/input.rs"]
pub(super) mod input;

#[path = "bridge_flow_coverage/rejections.rs"]
mod rejections;

#[test]
fn orca_parity_external_bridge_flow_thick_bounded_output() {
    compare("bridge_flow", true);
}

#[test]
fn orca_parity_external_bridge_flow_thin_bounded_output() {
    compare("bridge_flow", false);
}

#[test]
fn orca_parity_internal_bridge_flow_thick_bounded_output() {
    compare("internal_bridge_flow", true);
}

#[test]
fn orca_parity_internal_bridge_flow_thin_bounded_output() {
    compare("internal_bridge_flow", false);
}

fn compare(key: &str, thick: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == key)
        .unwrap();
    let role = if key == "bridge_flow" {
        "Bridge"
    } else {
        "Internal Bridge"
    };
    let root = parity::artifacts::root_from_env().unwrap();
    let evidence = root.join(format!("{key}-thick-{thick}-bounded.json"));
    let mut proofs = Vec::new();
    let mut amounts = Vec::new();
    for wire in ["0.5", "1", "1.5", "2"] {
        let exported = input::export(
            &runner,
            key,
            wire,
            thick,
            &format!("bounded-flow/{key}/{wire}/thick-{thick}"),
        )
        .unwrap();
        let sliced = runner.slice_case(&exported).unwrap();
        let native = std::str::from_utf8(&sliced.reference).unwrap();
        let amount = role_extrusion(native, role);
        assert!(
            amount > 0.0,
            "{key}={wire} thick={thick}: missing {role}; {root:?}"
        );
        let outcome = parity::compare_case(&sliced);
        proofs.push(
            json!({"requested":wire,"role":role,"positive_role_e":amount,
            "full_output_status":outcome.status,"detail":outcome.detail}),
        );
        parity::artifacts::overwrite(&evidence, &serde_json::to_vec_pretty(&proofs).unwrap())
            .unwrap();
        assert_eq!(
            outcome.status, "PASS",
            "{key}={wire} thick={thick}: {}; {root:?}",
            outcome.detail
        );
        amounts.push(amount);
    }
    assert!(
        amounts.windows(2).any(|pair| pair[0] != pair[1]),
        "flow must change actual {role} extrusion: {amounts:?}"
    );
    assert!(
        plan.cases.iter().all(|case| {
            let value = case
                .value
                .as_ref()
                .unwrap()
                .as_str()
                .unwrap()
                .parse::<f64>()
                .unwrap();
            proofs
                .iter()
                .any(|proof| proof["requested"].as_str().unwrap().parse::<f64>().unwrap() == value)
        }),
        "every declared request must have a full-output witness"
    );
    let outcome = OptionOutcome::executed(&plan, plan.cases.len(), Vec::new(), 0, None);
    parity::artifacts::overwrite(
        &evidence,
        &serde_json::to_vec_pretty(&json!({
        "key":key,"thick":thick,"domain":input::metadata(key, thick),
        "cases":proofs,"classification":outcome.status,
        "detail":outcome.detail}))
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
