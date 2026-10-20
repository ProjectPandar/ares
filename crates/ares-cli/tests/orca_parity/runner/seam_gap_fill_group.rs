//! Fill.cpp:275-305,1274-1279: clipping belongs to the grouped owner.

use std::{collections::BTreeSet, fs};

use ares_core::FloatOrPercent;
use serde_json::json;

use super::{case_runner, observe};

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    input: &'static str,
    sha: &'static str,
    global_percent: u8,
    local_gap: Option<&'static str>,
}

const CASES: [Case; 5] = [
    Case {
        name: "equal-density",
        input: "fill-group-flow-ratio-key/equal-density.3mf",
        sha: "89a9b7babcf153df4377c809064d6036668c3e4667bc727302771d6630fe339b",
        global_percent: 10,
        local_gap: None,
    },
    Case {
        name: "seam-equal",
        input: "seam-gap-fill-group-key/seam-equal.3mf",
        sha: "03cfd43caac4c9c5661076a414885ac6f8bceb6cb8414483f9845acf56ab078f",
        global_percent: 10,
        local_gap: Some("10%"),
    },
    Case {
        name: "seam20",
        input: "seam-gap-fill-group-key/seam20.3mf",
        sha: "93d3b3ffae0821609f1052388e4f0313e8a7e2fae3ac43657539984537932c08",
        global_percent: 10,
        local_gap: Some("20%"),
    },
    Case {
        name: "global-seam20",
        input: "seam-gap-fill-group-key/global-seam20.3mf",
        sha: "0dd52f9e193f76ea9aa531c2aeeff72727ea3f6bfa3842eb9188a3b9f667b14d",
        global_percent: 20,
        local_gap: None,
    },
    Case {
        name: "absolute-equal",
        input: "seam-gap-fill-group-key/absolute-equal.3mf",
        sha: "7620eafdbeb793c46cfcf7745e3e8c94d5bdbd76cd8f7e8aa07f5e2ccd6dde8b",
        global_percent: 10,
        local_gap: Some("0.04"),
    },
];

#[test]
fn baseline_control_preserves_complete_output() {
    check(CASES[0]);
}

#[test]
fn equal_percent_control_preserves_complete_output() {
    check(CASES[1]);
}

#[test]
fn local_seam_gap_matches_native_complete_output() {
    check(CASES[2]);
}

#[test]
fn global_seam_gap_control_preserves_complete_output() {
    check(CASES[3]);
}

#[test]
fn absolute_equal_control_preserves_complete_output() {
    check(CASES[4]);
}

fn check(case: Case) {
    case_runner::check(
        "seam-gap-fill-group-key",
        case_runner::Input {
            name: case.name,
            path: case.input,
            sha: case.sha,
        },
        |loaded, metadata| {
            assert!(matches!(
                loaded.settings().process.region.seam_gap,
                FloatOrPercent::Percent(value) if value.0 == f64::from(case.global_percent)
            ));
            let modifier = case_runner::modifier(metadata);
            match case.local_gap {
                Some(value) => {
                    assert!(
                        modifier
                            .contains(&format!("<metadata key=\"seam_gap\" value=\"{value}\"/>"))
                    )
                }
                None => assert!(!modifier.contains("key=\"seam_gap\"")),
            }
        },
        |run| {
            let reference = std::str::from_utf8(&run.native).unwrap();
            for footer in [
                "; CONFIG_BLOCK_END".to_owned(),
                "; nozzle_diameter = 0.4".to_owned(),
                "; bottom_surface_pattern = monotonic".to_owned(),
                "; sparse_infill_density = 15%".to_owned(),
                format!("; seam_gap = {}%", case.global_percent),
            ] {
                assert!(
                    reference.lines().any(|line| line == footer),
                    "Native effective footer: {footer}; {:?}",
                    run.root
                );
            }
            let native_bottom =
                observe::positive_layers(&observe::moves(&run.native, "Bottom surface"));
            let api_bottom =
                observe::positive_layers(&observe::moves(&run.api_bytes, "Bottom surface"));
            assert_eq!(native_bottom, BTreeSet::from([1]), "Native Bottom active");
            assert_eq!(api_bottom, BTreeSet::from([1]), "API Bottom active");
            fs::write(
                run.root.join("seam-gap-witness.json"),
                serde_json::to_vec_pretty(&json!({
                    "case": case.name, "input_path": case.input,
                    "global_percent": case.global_percent, "local_gap": case.local_gap,
                    "native_bottom_layers": native_bottom, "api_bottom_layers": api_bottom,
                }))
                .unwrap(),
            )
            .unwrap();
        },
    );
}
