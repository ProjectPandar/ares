//! MultiMaterialSegmentation.cpp1144–1168,2169–2181,2197–2207 at8500fcdcc.
//! Public success-required RED; failure inventory: painted-side-cut-spec.md.

use crate::runner::{
    native_api_cli::{NativeCase, sha},
    native_stream_case,
};
use serde_json::json;
use std::{env, fs, process::Command};

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
    model_moves: [u64; 2],
}

const TARGETS: [Witness; 3] = [
    Witness {
        label: "split-w1-d0",
        sha: "265d61e302c60ba42d384defbeaf938cb88205246c3606238654e8baa21dfcf3",
        width: "1",
        depth: "0",
        model_moves: [3995, 240],
    },
    Witness {
        label: "split-w1-d1p5",
        sha: "07d8496e37ee5f4fa43a2e28abf077c345a7deac395357f5ee24e109b9cc321e",
        width: "1",
        depth: "1.5",
        model_moves: [3961, 261],
    },
    Witness {
        label: "split-w0-d0p5",
        sha: "a73cba5ae616adaf741feedb318e50587910e5cdbeb9e53a53dcfd56fbd5a1bf",
        width: "0",
        depth: "0.5",
        model_moves: [3854, 519],
    },
];

const CONTROL: Witness = Witness {
    label: "painted-split-control",
    sha: "a18492a3cb74b68012e2d8c8975dafaa81267e77731cdab28411badeb4dc31cc",
    width: "0",
    depth: "0",
    model_moves: [3674, 900],
};

#[test]
fn orca_parity_painted_side_cut_width_only() {
    check(&TARGETS[0]);
}

#[test]
fn orca_parity_painted_side_cut_depth_over_width() {
    check(&TARGETS[1]);
}

#[test]
fn orca_parity_painted_side_cut_depth_only() {
    check(&TARGETS[2]);
}

#[test]
fn orca_parity_painted_side_cut_native_preflight() {
    let mut qualified = Vec::new();
    for case in TARGETS.iter().chain([&CONTROL]) {
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
    let repo = crate::runner::repo_root();
    if case.label != CONTROL.label {
        return fs::read(
            repo.join("tests/parity/painted-side-cut")
                .join(format!("{}.3mf", case.label)),
        )
        .unwrap();
    }
    // Reuse the frozen painted preparer; the public control is its existing test.
    let root = tempfile::Builder::new()
        .prefix("painted-side-cut-control-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let result = Command::new(env::var_os("ARES_PYTHON_BIN").unwrap())
        .arg(repo.join("tests/parity/paint-color/prepare.py"))
        .arg(root.join("input"))
        .output()
        .unwrap();
    fs::write(root.join("prepare.stdout"), &result.stdout).unwrap();
    fs::write(root.join("prepare.stderr"), &result.stderr).unwrap();
    assert!(result.status.success(), "fixture preparation: {root:?}");
    fs::read(root.join("input/painted-split.3mf")).unwrap()
}

fn assert_native(reference: &NativeCase, replay: &NativeCase, case: &Witness) {
    let expected = json!({
        "positive_model_XYE_by_tool":{"0":case.model_moves[0],"1":case.model_moves[1]},
        "both_material_positive_layers":48
    });
    let mut activities = Vec::new();
    for native in [reference, replay] {
        let observed = activity::activity(native.native.as_bytes(), &expected);
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
