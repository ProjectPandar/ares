//! PrintObject.cpp:4169-4237: region-local combination candidates.

use std::fs;

use ares_core::ProjectVolumeType;
use serde_json::json;

use super::{case_runner, observe};

#[derive(Clone, Copy)]
struct Case {
    input: case_runner::Input<'static>,
    combination: bool,
    maximum: &'static str,
    modifier: Option<(&'static str, &'static str)>,
}

const CASES: [Case; 7] = [
    Case {
        input: case_runner::Input {
            name: "combined-sparse",
            path: "combine-infill-region-view/combined-sparse.3mf",
            sha: "ca91af7af0bb5a20f631d423776d6fd0c9589a875a3bf6eb5077584dc69f38eb",
        },
        combination: true,
        maximum: "100%",
        modifier: Some(("seam_gap", "0.04")),
    },
    Case {
        input: case_runner::Input {
            name: "combined-density25",
            path: "combine-infill-region-view/combined-density25.3mf",
            sha: "be44e22474f9cb1a169175cc508c6a408f41004480ed49a5e0720a44b4df0afd",
        },
        combination: true,
        maximum: "100%",
        modifier: Some(("sparse_infill_density", "25%")),
    },
    Case {
        input: case_runner::Input {
            name: "canonical",
            path: "seam-gap-fill-group-key/absolute-equal.3mf",
            sha: "7620eafdbeb793c46cfcf7745e3e8c94d5bdbd76cd8f7e8aa07f5e2ccd6dde8b",
        },
        combination: false,
        maximum: "100%",
        modifier: Some(("seam_gap", "0.04")),
    },
    Case {
        input: case_runner::Input {
            name: "combined-identical",
            path: "combine-infill-region-view/combined-identical.3mf",
            sha: "f9acf0b75021da64743e389446a2af099fa647edba60217b3a8386396e6761d5",
        },
        combination: true,
        maximum: "100%",
        modifier: Some(("sparse_infill_density", "15%")),
    },
    Case {
        input: case_runner::Input {
            name: "combined-single",
            path: "combine-infill-region-view/combined-single.3mf",
            sha: "76fbbcdf669a9ba39c368c9a27724c865ced36761a315cb00274ad160ac4c6ad",
        },
        combination: true,
        maximum: "100%",
        modifier: None,
    },
    Case {
        input: case_runner::Input {
            name: "one-layer-distinct",
            path: "combine-infill-region-view/one-layer-distinct.3mf",
            sha: "0262510728204af2e99b869e3cf63ba687068d26c6704dbbb17c7261d3d89bcb",
        },
        combination: true,
        maximum: "0.2",
        modifier: Some(("seam_gap", "0.04")),
    },
    Case {
        input: case_runner::Input {
            name: "density25-without-combination",
            path: "combine-infill-region-view/density25-without-combination.3mf",
            sha: "c5a43f0c5a2e29b80e3c804b280098bb24c8c48fe6fb3426f8bc19b3e83475d7",
        },
        combination: false,
        maximum: "100%",
        modifier: Some(("sparse_infill_density", "25%")),
    },
];

#[test]
fn distinct_regions_combination_matches_native_complete_output() {
    check(CASES[0]);
}

#[test]
fn distinct_density_combination_matches_native_complete_output() {
    check(CASES[1]);
}

#[test]
fn disabled_combination_control_preserves_complete_output() {
    check(CASES[2]);
}

#[test]
fn identical_region_control_preserves_complete_output() {
    check(CASES[3]);
}

#[test]
fn single_region_control_preserves_complete_output() {
    check(CASES[4]);
}

#[test]
fn one_layer_gate_control_preserves_complete_output() {
    check(CASES[5]);
}

#[test]
fn disabled_density_control_preserves_complete_output() {
    check(CASES[6]);
}

fn check(case: Case) {
    case_runner::check(
        "combine-infill-region-view",
        case.input,
        |loaded, metadata| {
            assert_eq!(loaded.objects().len(), 1);
            let object = &loaded.objects()[0];
            assert_eq!(object.instances().len(), 1);
            assert_eq!(
                object.volumes().len(),
                1 + usize::from(case.modifier.is_some())
            );
            assert_eq!(
                object.volumes()[0].volume_type(),
                ProjectVolumeType::ModelPart
            );
            let region = &loaded.settings().process.region;
            assert_eq!(region.infill_combination.0, case.combination);
            assert_eq!(region.sparse_infill_density.0, 15.0);
            match case.modifier {
                Some((key, value)) => {
                    assert_eq!(
                        object.volumes()[1].volume_type(),
                        ProjectVolumeType::ParameterModifier
                    );
                    assert!(
                        case_runner::modifier(metadata)
                            .contains(&format!("<metadata key=\"{key}\" value=\"{value}\"/>"))
                    );
                }
                None => assert!(!metadata.contains("subtype=\"modifier_part\"")),
            }
        },
        |run| {
            let reference = std::str::from_utf8(&run.native).unwrap();
            for footer in [
                "; CONFIG_BLOCK_END".to_owned(),
                "; sparse_infill_density = 15%".to_owned(),
                "; sparse_infill_pattern = rectilinear".to_owned(),
                "; layer_height = 0.2".to_owned(),
                format!("; infill_combination = {}", u8::from(case.combination)),
                format!("; infill_combination_max_layer_height = {}", case.maximum),
            ] {
                assert!(
                    reference.lines().any(|line| line == footer),
                    "Native effective footer: {footer}; {:?}",
                    run.root
                );
            }
            let native_layers = observe::positive_layers(&run.native_sparse);
            let api_layers = observe::positive_layers(&run.api_sparse);
            assert!(!native_layers.is_empty(), "Native Sparse active");
            assert!(!api_layers.is_empty(), "API Sparse active");
            fs::write(
                run.root.join("combine-region-witness.json"),
                serde_json::to_vec_pretty(&json!({
                    "case": case.input.name, "input_path": case.input.path,
                    "combination": case.combination, "maximum": case.maximum,
                    "modifier": case.modifier, "native_sparse_layers": native_layers,
                    "api_sparse_layers": api_layers,
                }))
                .unwrap(),
            )
            .unwrap();
        },
    );
}
