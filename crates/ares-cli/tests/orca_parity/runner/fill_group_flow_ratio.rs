//! Fill.cpp:275-305,1012-1051: source ordered-set identity and first owner.

use std::{collections::BTreeSet, fs};

use ares_core::{ProcessInfillPattern, ProjectVolumeType};
use serde_json::json;

use super::{case_runner, observe};

#[derive(Clone, Copy)]
struct Case {
    name: &'static str,
    sha: &'static str,
    global_ratio: f64,
    local: Option<(&'static str, &'static str)>,
}

const CASES: [Case; 5] = [
    Case {
        name: "equal-density",
        sha: "89a9b7babcf153df4377c809064d6036668c3e4667bc727302771d6630fe339b",
        global_ratio: 1.0,
        local: None,
    },
    Case {
        name: "ratio-equal",
        sha: "887cb0a5f4409b3e386be772394496ef461e54cbda421f290f8d41ad4600a500",
        global_ratio: 1.0,
        local: Some(("top_solid_infill_flow_ratio", "1")),
    },
    Case {
        name: "ratio11",
        sha: "38fd0b9767a231f736e5a0a0b70fab3fdf1d6c411ff524c60f2ab63b14d1166c",
        global_ratio: 1.0,
        local: Some(("top_solid_infill_flow_ratio", "1.1")),
    },
    Case {
        name: "global-ratio11",
        sha: "73efbf93cd3043c2a7e98e23ed82d6fb806c2f590015e20475ad205fbbcefc7d",
        global_ratio: 1.1,
        local: None,
    },
    Case {
        name: "local-speed45",
        sha: "2f403750506a532562fa6d97adeb801587a648ab37037feef3e3fc574786ce4a",
        global_ratio: 1.0,
        local: Some(("top_surface_speed", "45")),
    },
];

#[test]
fn equal_density_control_matches_native_complete_output() {
    check(CASES[0]);
}

#[test]
fn local_equal_ratio_control_matches_native_complete_output() {
    check(CASES[1]);
}

#[test]
fn local_top_flow_ratio_matches_native_complete_output() {
    check(CASES[2]);
}

#[test]
fn global_top_flow_ratio_control_matches_native_complete_output() {
    check(CASES[3]);
}

#[test]
fn local_top_speed_control_matches_native_complete_output() {
    check(CASES[4]);
}

fn check(case: Case) {
    let path = format!("fill-group-flow-ratio-key/{}.3mf", case.name);
    case_runner::check(
        "fill-group-flow-ratio-key",
        case_runner::Input {
            name: case.name,
            path: &path,
            sha: case.sha,
        },
        |loaded, metadata| {
            assert_eq!(loaded.objects().len(), 1);
            let object = &loaded.objects()[0];
            assert_eq!(object.instances().len(), 1);
            assert_eq!(
                object
                    .volumes()
                    .iter()
                    .map(|v| v.volume_type())
                    .collect::<Vec<_>>(),
                [
                    ProjectVolumeType::ModelPart,
                    ProjectVolumeType::ParameterModifier
                ]
            );
            let region = &loaded.settings().process.region;
            assert_eq!(region.sparse_infill_density.0, 15.0);
            assert_eq!(
                region.sparse_infill_pattern,
                ProcessInfillPattern::Rectilinear
            );
            assert_eq!(region.top_solid_infill_flow_ratio.0, case.global_ratio);
            assert_eq!(region.top_surface_speed.0, 30.0);
            let modifier = case_runner::modifier(metadata);
            assert!(modifier.contains("<metadata key=\"sparse_infill_density\" value=\"15%\"/>"));
            for key in ["top_solid_infill_flow_ratio", "top_surface_speed"] {
                if let Some((local_key, value)) =
                    case.local.filter(|(local_key, _)| *local_key == key)
                {
                    assert!(modifier.contains(&format!(
                        "<metadata key=\"{local_key}\" value=\"{value}\"/>"
                    )));
                } else {
                    assert!(!modifier.contains(&format!("key=\"{key}\"")));
                }
            }
        },
        |run| {
            let reference = std::str::from_utf8(&run.native).unwrap();
            for footer in [
                "; CONFIG_BLOCK_END".to_owned(),
                "; sparse_infill_density = 15%".to_owned(),
                "; sparse_infill_pattern = rectilinear".to_owned(),
                "; top_surface_pattern = monotonicline".to_owned(),
                "; top_surface_speed = 30".to_owned(),
                format!("; top_solid_infill_flow_ratio = {}", case.global_ratio),
            ] {
                assert!(
                    reference.lines().any(|line| line == footer),
                    "Native effective footer: {footer}; {:?}",
                    run.root
                );
            }
            let native_top = observe::positive_layers(&observe::moves(&run.native, "Top surface"));
            let api_top = observe::positive_layers(&observe::moves(&run.api_bytes, "Top surface"));
            assert_eq!(native_top, BTreeSet::from([75]), "Native top active");
            assert_eq!(api_top, BTreeSet::from([75]), "API top active");
            fs::write(
                run.root.join("flow-ratio-witness.json"),
                serde_json::to_vec_pretty(&json!({
                    "case": case.name, "global_ratio": case.global_ratio, "local_override": case.local,
                    "native_top_layers": native_top, "api_top_layers": api_top,
                }))
                .unwrap(),
            )
            .unwrap();
        },
    );
}
