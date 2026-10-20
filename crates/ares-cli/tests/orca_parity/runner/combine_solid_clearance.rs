//! PrintObject.cpp:4252-4264: scaled-width clearance before float multiplication.

use std::fs;

use ares_core::{ProcessInfillPattern, ProjectVolumeType};
use serde_json::json;

use super::{case_runner, observe};

#[derive(Clone, Copy)]
struct Case {
    input: case_runner::Input<'static>,
    density: f64,
    combination: bool,
    pattern: ProcessInfillPattern,
    pattern_key: &'static str,
    maximum: &'static str,
}

const CASES: [Case; 6] = [
    Case {
        input: case_runner::Input {
            name: "sparse15-on",
            path: "combine-infill-region-view/combined-single.3mf",
            sha: "76fbbcdf669a9ba39c368c9a27724c865ced36761a315cb00274ad160ac4c6ad",
        },
        density: 15.0,
        combination: true,
        pattern: ProcessInfillPattern::Monotonic,
        pattern_key: "monotonic",
        maximum: "100%",
    },
    Case {
        input: case_runner::Input {
            name: "solid100-off",
            path: "combine-solid-clearance/solid100-off.3mf",
            sha: "c7a885fc01d6d0e28921a377aae9e3d4310cb258e6a32d8e5f1216cad4e9f3fd",
        },
        density: 100.0,
        combination: false,
        pattern: ProcessInfillPattern::Monotonic,
        pattern_key: "monotonic",
        maximum: "100%",
    },
    Case {
        input: case_runner::Input {
            name: "solid100-on",
            path: "combine-solid-clearance/solid100-on.3mf",
            sha: "4c4403fcd748643b7831be80ec8ea5f0856bcc069d21a70ff28a2a64e8861163",
        },
        density: 100.0,
        combination: true,
        pattern: ProcessInfillPattern::Monotonic,
        pattern_key: "monotonic",
        maximum: "100%",
    },
    Case {
        input: case_runner::Input {
            name: "solid100-rect",
            path: "combine-solid-clearance/solid100-rect.3mf",
            sha: "e5cf9a53a47f9e3a4413a4b2f3f81895fc9bf48b08a5993151e17c4a409458b9",
        },
        density: 100.0,
        combination: true,
        pattern: ProcessInfillPattern::Rectilinear,
        pattern_key: "rectilinear",
        maximum: "100%",
    },
    Case {
        input: case_runner::Input {
            name: "solid100-concentric",
            path: "combine-solid-clearance/solid100-concentric.3mf",
            sha: "3bbbe5853630542b8671521e35c1dde38f763ffef4f133f904eb25ee7aa76a9e",
        },
        density: 100.0,
        combination: true,
        pattern: ProcessInfillPattern::Concentric,
        pattern_key: "concentric",
        maximum: "100%",
    },
    Case {
        input: case_runner::Input {
            name: "solid100-one-layer",
            path: "combine-solid-clearance/solid100-one-layer.3mf",
            sha: "80910d21602bb78ca36df0f5771064b041d768016294deccfdbdf4c60c251791",
        },
        density: 100.0,
        combination: true,
        pattern: ProcessInfillPattern::Monotonic,
        pattern_key: "monotonic",
        maximum: "0.2",
    },
];

#[test]
fn sparse_combination_control_preserves_complete_output() {
    check(CASES[0]);
}

#[test]
fn disabled_solid_combination_control_preserves_complete_output() {
    check(CASES[1]);
}

#[test]
fn combined_monotonic_solid_matches_native_complete_output() {
    check(CASES[2]);
}

#[test]
fn combined_rectilinear_solid_matches_native_complete_output() {
    check(CASES[3]);
}

#[test]
fn combined_concentric_solid_matches_native_complete_output() {
    check(CASES[4]);
}

#[test]
fn one_layer_solid_control_preserves_complete_output() {
    check(CASES[5]);
}

fn check(case: Case) {
    case_runner::check(
        "combine-solid-clearance",
        case.input,
        |loaded, metadata| {
            assert_eq!(loaded.objects().len(), 1);
            let object = &loaded.objects()[0];
            assert_eq!(object.instances().len(), 1);
            assert_eq!(object.volumes().len(), 1);
            assert_eq!(
                object.volumes()[0].volume_type(),
                ProjectVolumeType::ModelPart
            );
            assert!(!metadata.contains("subtype=\"modifier_part\""));
            let region = &loaded.settings().process.region;
            assert_eq!(region.sparse_infill_density.0, case.density);
            assert_eq!(region.infill_combination.0, case.combination);
            assert_eq!(region.internal_solid_infill_pattern, case.pattern);
        },
        |run| {
            let reference = std::str::from_utf8(&run.native).unwrap();
            for footer in [
                "; CONFIG_BLOCK_END".to_owned(),
                "; layer_height = 0.2".to_owned(),
                format!("; sparse_infill_density = {}%", case.density),
                format!("; infill_combination = {}", u8::from(case.combination)),
                format!("; internal_solid_infill_pattern = {}", case.pattern_key),
                format!("; infill_combination_max_layer_height = {}", case.maximum),
            ] {
                assert!(
                    reference.lines().any(|line| line == footer),
                    "Native effective footer: {footer}; {:?}",
                    run.root
                );
            }
            let native_solid = observe::moves(&run.native, "Internal solid infill");
            let api_solid = observe::moves(&run.api_bytes, "Internal solid infill");
            assert!(!native_solid.is_empty(), "Native solid active");
            assert!(!api_solid.is_empty(), "API solid active");
            assert_eq!(run.native_sparse.is_empty(), case.density == 100.0);
            assert_eq!(run.api_sparse.is_empty(), case.density == 100.0);
            let combined_height = case.combination && case.maximum == "100%";
            for bytes in [&run.native, &run.api_bytes] {
                let text = std::str::from_utf8(bytes).unwrap();
                assert_eq!(
                    text.lines().any(|line| line == ";HEIGHT:0.4"),
                    combined_height,
                    "combination height active: {:?}",
                    run.root
                );
            }
            fs::write(
                run.root.join("combine-clearance-witness.json"),
                serde_json::to_vec_pretty(&json!({
                    "case": case.input.name, "input_path": case.input.path,
                    "density": case.density, "combination": case.combination,
                    "pattern": case.pattern_key, "maximum": case.maximum,
                    "combined_height": combined_height,
                    "native_solid_layers": observe::positive_layers(&native_solid),
                    "api_solid_layers": observe::positive_layers(&api_solid),
                    "native_solid_moves": native_solid.len(),
                    "api_solid_moves": api_solid.len(),
                    "native_sparse_layers": observe::positive_layers(&run.native_sparse),
                    "api_sparse_layers": observe::positive_layers(&run.api_sparse),
                }))
                .unwrap(),
            )
            .unwrap();
        },
    );
}
