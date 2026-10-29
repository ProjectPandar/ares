//! PrintObjectSlice.cpp1033–1141,1227–1241 / FuzzySkin.cpp561–684 at8500fcdcc.
//! Success-required public RED; failure inventory: painted-fuzzy-skin-spec.md.

use crate::runner::{
    native_api_cli::{NativeCase, sha},
    native_stream_case,
};
use serde_json::{Value, json};
use std::fs;

const UPSTREAM: &[&str] = &[
    "Format/bbs_3mf.cpp:330,3824,5058-5078,7339-7346",
    "PrintApply.cpp:1081-1110",
    "MultiMaterialSegmentation.cpp:1955-2194,2211-2226",
    "PrintObjectSlice.cpp:1033-1141,1227-1243",
    "Feature/FuzzySkin/FuzzySkin.cpp:415-473,502-684",
];
const CASES: [(&str, &str); 2] = [
    (
        "stock-fuzzy-unpainted",
        "603c3228513e194c6fccabea843ba3640b3b96aefa7b4728ea902c9f59fa8357",
    ),
    (
        "stock-fuzzy-painted-face",
        "3d3401362eb9904e84999b2e2ab6368137e521702f4ebb26dec779608ada0f22",
    ),
];

fn input(label: &str) -> Vec<u8> {
    fs::read(
        crate::runner::repo_root()
            .join("tests/parity/painted-fuzzy")
            .join(format!("{label}.3mf")),
    )
    .unwrap()
}

// Monofilament stock G-code need not emit T0; do not use the MMU tool counter.
// This is only an activity diagnostic; NativeCase compares complete ordered bytes.
fn wall_activity(gcode: &str) -> Value {
    let mut relative = false;
    let mut role = "";
    let mut walls = Vec::new();
    for line in gcode.lines() {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        let mut words = line.split(';').next().unwrap().split_whitespace();
        match words.next() {
            Some("M83") => relative = true,
            Some("M82") => relative = false,
            Some("G1") if matches!(role, "Outer wall" | "Inner wall") => {
                let fields: Vec<_> = words.collect();
                if fields.iter().any(|word| word.starts_with(['X', 'Y']))
                    && fields
                        .iter()
                        .find_map(|word| word.strip_prefix('E'))
                        .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
                {
                    assert!(relative, "stock witness must use relative extrusion");
                    walls.push(line);
                }
            }
            _ => {}
        }
    }
    assert!(!walls.is_empty(), "Native must extrude model walls with XY");
    json!({"positive_wall_XYE": walls.len(),
        "ordered_positive_wall_XYE_sha256": sha(walls.join("\n").as_bytes())})
}

fn assert_native(reference: &NativeCase, replay: &NativeCase) -> Value {
    let mut observed = Vec::new();
    for native in [reference, replay] {
        for (key, value) in [
            ("wall_generator", "classic"),
            ("fuzzy_skin", "none"),
            ("fuzzy_skin_noise_type", "ripple"),
            ("fuzzy_skin_thickness", "0.2"),
            ("fuzzy_skin_point_distance", "0.3"),
            ("fuzzy_skin_first_layer", "0"),
            ("fuzzy_skin_mode", "displacement"),
            ("fuzzy_skin_ripples_per_layer", "15"),
            ("fuzzy_skin_ripple_offset", "50%"),
            ("fuzzy_skin_layers_between_ripple_offset", "1"),
            ("use_relative_e_distances", "1"),
            ("filament_diameter", "1.75"),
            ("enable_support", "0"),
            ("enable_prime_tower", "0"),
            ("interlocking_beam", "0"),
            ("xy_hole_compensation", "0"),
            ("xy_contour_compensation", "0"),
            ("elefant_foot_compensation", "0.1"),
        ] {
            assert!(native.native.contains(&format!("; {key} = {value}\n")));
        }
        let activity = wall_activity(&native.native);
        fs::write(
            native.root.join("native-activity.json"),
            serde_json::to_vec_pretty(&activity).unwrap(),
        )
        .unwrap();
        observed.push(activity);
    }
    assert_eq!(observed[0], observed[1], "Native wall activity replay");
    observed.remove(0)
}

#[test]
fn orca_parity_painted_fuzzy_native_preflight() {
    let mut qualified = Vec::new();
    let mut activities = Vec::new();
    for (label, digest) in CASES {
        let project = input(label);
        let (reference, replay) = native_stream_case::qualify(
            ("painted-fuzzy-preflight", UPSTREAM),
            label,
            &project,
            digest,
            false,
        );
        activities.push(assert_native(&reference, &replay));
        qualified.push(json!({"case":label,"input_sha256":sha(&project),
            "reference":reference.root,"replay":replay.root}));
    }
    assert_ne!(
        activities[0]["ordered_positive_wall_XYE_sha256"],
        activities[1]["ordered_positive_wall_XYE_sha256"],
        "Native painting must change positive model wall motion"
    );
    fs::write(
        crate::artifacts::root_from_env()
            .unwrap()
            .join("painted-fuzzy-preflight.json"),
        serde_json::to_vec_pretty(&qualified).unwrap(),
    )
    .unwrap();
}

fn check(index: usize) {
    let (label, digest) = CASES[index];
    native_stream_case::check(
        ("painted-fuzzy", UPSTREAM),
        label,
        &input(label),
        digest,
        (false, |reference, replay| {
            assert_native(reference, replay);
        }),
    );
}

#[test]
fn orca_parity_painted_fuzzy_unpainted_control() {
    check(0);
}

#[test]
fn orca_parity_painted_fuzzy_painted_face() {
    check(1);
}
