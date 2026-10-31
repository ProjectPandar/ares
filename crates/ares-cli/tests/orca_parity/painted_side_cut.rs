//! MultiMaterialSegmentation.cpp1144–1168,2169–2181,2197–2207 at8500fcdcc.
//! Public success-required RED; failure inventory: painted-side-cut-spec.md.

use crate::runner::{
    native_api_cli::{NativeCase, sha},
    native_stream_case,
};
use serde_json::json;
use std::fs;

use crate::paint_color_activity as activity;

const UPSTREAM: &[&str] = &[
    "MultiMaterialSegmentation.cpp:1144-1168,2169-2181,2197-2207",
    "PrintObjectSlice.cpp:874-886,1206-1224,1243-1254",
    "PrintConfig.cpp:4206-4224;libslic3r.h:93;ClipperUtils.hpp:19,27,346,449",
];

struct Witness {
    label: &'static str,
    sha: &'static str,
    width: &'static str,
    depth: &'static str,
}

const TARGETS: [Witness; 3] = [
    Witness {
        label: "split-d0",
        sha: "265d61e302c60ba42d384defbeaf938cb88205246c3606238654e8baa21dfcf3",
        width: "1",
        depth: "0",
    },
    Witness {
        label: "split-d1p5",
        sha: "07d8496e37ee5f4fa43a2e28abf077c345a7deac395357f5ee24e109b9cc321e",
        width: "1",
        depth: "1.5",
    },
    Witness {
        label: "split-d0p5",
        sha: "a73cba5ae616adaf741feedb318e50587910e5cdbeb9e53a53dcfd56fbd5a1bf",
        width: "0",
        depth: "0.5",
    },
];

#[test]
fn painted_side_cut_width_only() {
    check(&TARGETS[0]);
}

#[test]
fn painted_side_cut_depth_over_width() {
    check(&TARGETS[1]);
}

#[test]
fn painted_side_cut_depth_only() {
    check(&TARGETS[2]);
}

#[test]
fn painted_side_cut_native_preflight() {
    let mut qualified = Vec::new();
    for case in &TARGETS {
        let project = input(case);
        let (reference, replay) = native_stream_case::qualify(
            ("painted-side-cut-preflight", UPSTREAM),
            case.label,
            &project,
            case.sha,
            true,
        );
        assert_native(&reference, &replay, case);
        qualified.push(json!({"case":case.label,"input_sha256":sha(&project),
            "reference":reference.root,"replay":replay.root}));
    }
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join("painted-side-cut-preflight.json"),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}

fn check(case: &Witness) {
    let project = input(case);
    native_stream_case::check(
        ("painted-side-cut", UPSTREAM),
        case.label,
        &project,
        case.sha,
        (true, |reference, replay| {
            assert_native(reference, replay, case)
        }),
    );
}

fn input(case: &Witness) -> Vec<u8> {
    fs::read(
        crate::runner::repo_root()
            .join("tests/parity/painted-side-cut")
            .join(format!("{}.3mf", case.label)),
    )
    .unwrap()
}

fn assert_native(reference: &NativeCase, replay: &NativeCase, case: &Witness) {
    let mut activities = Vec::new();
    for native in [reference, replay] {
        let observed = activity::activity(native.native.as_bytes());
        for tool in ["0", "1"] {
            assert!(
                observed["positive_model_XYE_by_tool"][tool]
                    .as_u64()
                    .unwrap()
                    > 0
            );
        }
        fs::write(
            native.root.join("native-activity.json"),
            serde_json::to_vec_pretty(&observed).unwrap(),
        )
        .unwrap();
        for (key, value) in [
            ("mmu_segmented_region_max_width", case.width),
            ("mmu_segmented_region_interlocking_depth", case.depth),
            ("interlocking_beam", "0"),
            ("enable_prime_tower", "0"),
        ] {
            assert!(native.native.contains(&format!("; {key} = {value}\n")));
        }
        activities.push(observed);
    }
    assert_eq!(activities[0], activities[1], "Native model activity replay");
}
