//! Native-backed sequence admission; baseline-only count classification stays RED.
use super::*;
use serde_json::json;

#[path = "tool_sequence_context.rs"]
mod context;
#[path = "tool_sequence_observation.rs"]
mod observation;

struct SequenceCase {
    name: &'static str,
    count: usize,
    vector: &'static [usize],
    expected: &'static [(usize, [usize; 2])],
}

pub(super) fn execute_plan(runner: &OrcaRunner, plan: &domains::OptionPlan) -> OptionOutcome {
    let root = parity::artifacts::root_from_env().unwrap();
    let mut proofs = Vec::new();
    for case in &plan.cases {
        let count: usize = case
            .value
            .as_ref()
            .unwrap()
            .as_str()
            .unwrap()
            .parse()
            .unwrap();
        let vector: &[usize] = match count {
            0 | 1 => &[2, 5, 2, 1],
            2 => &[2, 5, 1, 2, 4, 6, 2, 1],
            _ => unreachable!("bounded admitted count plan"),
        };
        let expected: &[(usize, [usize; 2])] = match count {
            0 => &[],
            1 => &[(2, [1, 0]), (3, [1, 0]), (4, [1, 0]), (5, [1, 0])],
            2 => &[
                (2, [0, 1]),
                (3, [0, 1]),
                (4, [1, 0]),
                (5, [1, 0]),
                (6, [1, 0]),
            ],
            _ => unreachable!("bounded admitted count plan"),
        };
        let label = format!("option/{}/{}", plan.key, case.label);
        let exported = context::prepare(runner, &label, count, vector);
        let sliced = runner.slice_case(&exported).unwrap();
        let native = observation::observe(std::str::from_utf8(&sliced.reference).unwrap());
        let outcome = parity::compare_case(&sliced);
        let ares = observation::observe(
            &std::fs::read_to_string(outcome.artifacts.as_ref().unwrap().join("ares.gcode"))
                .unwrap(),
        );
        proofs.push(json!({"label":label,"count":count,"vector":vector,
            "native":native,"ares":ares,"full_output_status":outcome.status,
            "artifacts":outcome.artifacts,"detail":outcome.detail}));
        parity::artifacts::overwrite(
            &root.join("g4-sequence-sweep-cases.json"),
            &serde_json::to_vec_pretty(&proofs).unwrap(),
        )
        .unwrap();
        assert_eq!(outcome.status, "PASS", "{label}: {}", outcome.detail);
        observation::verify(&native, &label, expected);
        observation::verify(&ares, &label, expected);
        assert_eq!(native, ares, "{label}: actual per-layer consumption");
    }
    OptionOutcome::executed(plan, proofs.len(), Vec::new(), 0, None)
}

#[test]
fn orca_parity_g4_other_layer_sequence_count_consumes_ranges_and_last_row_priority() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let plan = domains::load(&runner::repo_root())
        .into_iter()
        .find(|plan| plan.key == "other_layers_print_sequence_nums")
        .unwrap();
    let cases = [
        SequenceCase {
            name: "count0",
            count: 0,
            vector: &[2, 5, 2, 1],
            expected: &[],
        },
        SequenceCase {
            name: "count1",
            count: 1,
            vector: &[2, 5, 2, 1],
            expected: &[(2, [1, 0]), (3, [1, 0]), (4, [1, 0]), (5, [1, 0])],
        },
        SequenceCase {
            name: "count2",
            count: 2,
            vector: &[2, 5, 1, 2, 4, 6, 2, 1],
            expected: &[
                (2, [0, 1]),
                (3, [0, 1]),
                (4, [1, 0]),
                (5, [1, 0]),
                (6, [1, 0]),
            ],
        },
        SequenceCase {
            name: "empty-control",
            count: 1,
            vector: &[],
            expected: &[],
        },
        SequenceCase {
            name: "reverse-priority-control",
            count: 2,
            vector: &[4, 6, 2, 1, 2, 5, 1, 2],
            expected: &[
                (2, [0, 1]),
                (3, [0, 1]),
                (4, [0, 1]),
                (5, [0, 1]),
                (6, [1, 0]),
            ],
        },
    ];
    let root = parity::artifacts::root_from_env().unwrap();
    let mut proofs = Vec::new();
    for SequenceCase {
        name,
        count,
        vector,
        expected,
    } in cases
    {
        let label = format!("g4-sequences/{name}");
        let exported = context::prepare(&runner, &label, count, vector);
        let sliced = runner.slice_case(&exported).unwrap();
        let native = observation::observe(std::str::from_utf8(&sliced.reference).unwrap());
        let outcome = parity::compare_case(&sliced);
        let ares_path = outcome.artifacts.as_ref().unwrap().join("ares.gcode");
        let ares = observation::observe(&std::fs::read_to_string(&ares_path).unwrap());
        proofs.push(json!({
            "label":label,"count":count,"vector":vector,
            "canonical":matches!(name,"count0"|"count1"|"count2"),
            "native":native,"ares":ares,"expected_range_orders":expected,
            "full_output_status":outcome.status,"detail":outcome.detail,"artifacts":outcome.artifacts
        }));
        parity::artifacts::overwrite(
            &root.join("g4-sequence-cases.json"),
            &serde_json::to_vec_pretty(&proofs).unwrap(),
        )
        .unwrap();
        assert_eq!(outcome.status, "PASS", "{label}: {}", outcome.detail);
        observation::verify(&native, &label, expected);
        observation::verify(&ares, &label, expected);
        assert_eq!(native, ares, "{label}: actual per-layer consumption");
    }
    assert_eq!(
        proofs[0]["native"], proofs[3]["native"],
        "count0 and empty vector both disable custom ranges"
    );
    assert_ne!(
        proofs[2]["native"]["ordered_extrusion_runs"]["4"],
        proofs[4]["native"]["ordered_extrusion_runs"]["4"],
        "overlap row precedence must be consumed"
    );
    let outcome = OptionOutcome::executed(&plan, 3, Vec::new(), 0, None);
    parity::artifacts::overwrite(&root.join("g4-sequence-bounded.json"), &serde_json::to_vec_pretty(&json!({
        "canonical_count":3,"controls":2,"classification":outcome.status,"detail":outcome.detail,
        "source":"ParameterUtils.cpp:6-29; ToolOrdering.cpp:1320-1362",
        "scope":"one-nozzle, two logical filaments, by-layer coherent count0/1/2",
        "deferred":["general count domain","malformed vectors","by-object","other G4 keys"]
    })).unwrap()).unwrap();
    // This is the intended metadata RED only if every actual Native consumer
    // witness and unchanged whole comparator above has already passed.
    assert_eq!(
        outcome.status, "PASS",
        "other_layers_print_sequence_nums: {}",
        outcome.detail
    );
    assert!(plan.provably_legal);
    assert!(plan.omission.is_none());
    assert_eq!(
        plan.cases
            .iter()
            .map(|case| case.value.clone().unwrap())
            .collect::<Vec<_>>(),
        [json!("0"), json!("1"), json!("2")]
    );
}
