//! PrintConfig.cpp:5068-5079; OrcaSlicer.cpp:1682-1690,6247-6248.
use super::{OptionOutcome, domains, parity, runner::OrcaRunner};
use serde_json::{Value, json};

const CASES: [(&str, &[&str]); 3] = [
    ("empty-vector", &[]),
    ("empty-string-vector", &[""]),
    (
        "owned-script",
        &["<owned absolute script path resolved in comparison receipt>"],
    ),
];

pub(super) fn generate() -> (Vec<domains::OptionCase>, Value) {
    let cases = CASES
        .iter()
        .map(|(label, requested)| domains::OptionCase {
            label: (*label).into(),
            value: Some(json!(requested)),
        })
        .collect();
    (
        cases,
        json!({
            "domain":"bounded embedded ordinary Linux CLI admission, not script execution",
            "source":"PrintConfig.cpp:5068-5079; OrcaSlicer.cpp:1682-1690,6247-6248",
            "spec":"docs/architecture/2026-10-14-post-process-matrix-admission-spec.md",
            "cases":CASES.iter().map(|(label, requested)| json!({
                "label":label,"request_template":requested,
                "resolution":if *label == "owned-script" { "comparison receipt records concrete owned absolute script vector" } else { "literal embedded vector" }
            })).collect::<Vec<_>>(),
            "comparison_count_semantics":"one strict whole G-code plus two expected exact CLI rejection comparisons",
            "deferred":["external execution","arbitrary coStrings","load-settings composition","normative_check=false","other platforms","additional whitespace/multi-entry cases"]
        }),
    )
}

pub(super) fn execute_plan(runner: &OrcaRunner, plan: &domains::OptionPlan) -> OptionOutcome {
    let root = parity::artifacts::root_from_env().unwrap();
    parity::artifacts::overwrite(
        &root.join("post-process-domain.json"),
        &serde_json::to_vec_pretty(plan.tool_metadata.as_ref().unwrap()).unwrap(),
    )
    .unwrap();
    let mut strict_gcode = 0;
    let mut expected_cli_rejection = 0;
    for request in &plan.cases {
        let &(label, template) = CASES
            .iter()
            .find(|(label, _)| *label == request.label)
            .unwrap();
        assert_eq!(request.value, Some(json!(template)));
        let scripts = if label == "owned-script" {
            None
        } else {
            request.value.clone()
        };
        let (directory, requested) = crate::post_process_cli::compare_with(runner, scripts);
        let rejected = !requested.as_array().unwrap().is_empty();
        let kind = if rejected {
            expected_cli_rejection += 1;
            "expected CLI rejection"
        } else {
            strict_gcode += 1;
            "strict whole G-code"
        };
        let mut receipt = tempfile::NamedTempFile::new_in(&root).unwrap();
        serde_json::to_writer_pretty(
            receipt.as_file_mut(),
            &json!({
                "key":plan.key,"label":label,"directory":directory,
                "requested":requested,"comparison":"PASS","kind":kind,
                "expected_exit":if rejected {237} else {0},
                "execution_marker_absent":!directory.join("must-not-run.sh.marker").exists()
            }),
        )
        .unwrap();
        receipt
            .persist(root.join(format!("post-process-{label}-comparison.json")))
            .unwrap();
    }
    // The sweep and its public regression can publish the same bounded counts.
    let mut counts = tempfile::NamedTempFile::new_in(&root).unwrap();
    serde_json::to_writer_pretty(
        counts.as_file_mut(),
        &json!({
            "key":plan.key,"strict_gcode":strict_gcode,
            "expected_cli_rejection":expected_cli_rejection,
            "scope":"embedded CLI admission only; external execution uncovered"
        }),
    )
    .unwrap();
    counts
        .persist(root.join("post-process-comparison-counts.json"))
        .unwrap();
    let mut outcome = OptionOutcome::executed(plan, plan.cases.len(), Vec::new(), 0, None);
    if !outcome.detail.is_empty() {
        outcome.detail.push_str("; ");
    }
    outcome.detail.push_str(&format!(
        "bounded embedded CLI admission: {strict_gcode} strict G-code + {expected_cli_rejection} expected CLI rejection comparisons; external execution uncovered"
    ));
    outcome
}
