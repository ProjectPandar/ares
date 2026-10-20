//! Print.cpp:2794-2990 / GCode.cpp:4465-4540: AutoBrim occupied skirt groups.

use crate::{
    OrcaRunner,
    runner::native_api_cli::{NativeCase, sha},
};

#[test]
fn perobject_auto_admission_far() {
    run(
        "far-perobject-auto",
        include_bytes!(
            "../../../../tests/parity/per-object-auto-brim-admission/far-perobject-auto.3mf"
        ),
        "339a797d71afc111b8857aae4db0ef405eb6a1534f1404a9a6762395651d4390",
        &["Skirt", "Brim", "Outer wall", "Skirt", "Brim", "Outer wall"],
    );
}

#[test]
fn perobject_auto_admission_near() {
    run(
        "near-perobject-auto",
        include_bytes!(
            "../../../../tests/parity/per-object-auto-brim-admission/near-perobject-auto.3mf"
        ),
        "33879016885adcc85533e345a52a46d6ef4fbe74dd63156305f07bc1981d2d6a",
        &["Skirt", "Brim", "Outer wall", "Brim", "Outer wall"],
    );
}

#[test]
fn perobject_auto_admission_far_combined_control() {
    run(
        "far-combined-auto",
        include_bytes!(
            "../../../../tests/parity/per-object-auto-brim-admission/far-combined-auto.3mf"
        ),
        "ee5a89805ca47d75c0268015f393c4a60f8c278ab761b25a94ca1f88acd9a4a1",
        &["Skirt", "Brim", "Outer wall", "Brim", "Outer wall"],
    );
}

#[test]
fn perobject_auto_admission_near_combined_control() {
    run(
        "near-combined-auto",
        include_bytes!(
            "../../../../tests/parity/per-object-auto-brim-admission/near-combined-auto.3mf"
        ),
        "686575c34b793f4f872f12737de583e8a1f579b025b54dfbca1db621b57c5736",
        &["Skirt", "Brim", "Outer wall", "Brim", "Outer wall"],
    );
}

#[test]
fn perobject_auto_admission_far_no_brim_control() {
    run(
        "far-perobject-none",
        include_bytes!(
            "../../../../tests/parity/per-object-auto-brim-admission/far-perobject-none.3mf"
        ),
        "b64e6d245c5df8e92a12df954f0f7caa1939ff1ee6235c58a2374b770cf7024e",
        &["Skirt", "Outer wall", "Skirt", "Outer wall"],
    );
}

#[test]
fn perobject_auto_admission_near_no_brim_control() {
    run(
        "near-perobject-none",
        include_bytes!(
            "../../../../tests/parity/per-object-auto-brim-admission/near-perobject-none.3mf"
        ),
        "e22bfef922c343c7ba2e91514191c851ee9dd8ee89f11cd9e23cdf7e42e3baea",
        &["Skirt", "Outer wall", "Skirt", "Outer wall"],
    );
}

fn run(label: &str, project: &[u8], expected_sha: &str, expected_roles: &[&str]) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    assert_eq!(sha(project), expected_sha);
    let case = NativeCase::slice(&runner, project, &format!("{label}-"));
    assert_footprint(case.native.as_bytes(), expected_roles);
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 10, 20, 0, 0, 0,
    ));
    assert_footprint(&api, expected_roles);
    assert_footprint(&cli, expected_roles);
}

fn assert_footprint(bytes: &[u8], expected_roles: &[&str]) {
    let text = std::str::from_utf8(bytes).unwrap();
    let first_layer = text.split(";LAYER_CHANGE").nth(1).unwrap();
    let roles = first_layer
        .lines()
        .filter_map(|line| line.strip_prefix(";TYPE:"))
        .collect::<Vec<_>>();
    assert_eq!(roles, expected_roles);
    let mut role = "";
    let mut brim_moves = 0;
    let mut skirt_moves = 0;
    // These frozen projects select relative extrusion (M83). Count actual
    // positive-E XY moves, not only TYPE comments or retraction/unretraction.
    assert!(
        text.lines()
            .any(|line| line.split_ascii_whitespace().next() == Some("M83"))
    );
    for line in first_layer.lines() {
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        let words = line.split(';').next().unwrap().split_ascii_whitespace();
        let positive_xy = line.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.clone().any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            });
        if positive_xy {
            match role {
                "Brim" => brim_moves += 1,
                "Skirt" => skirt_moves += 1,
                _ => {}
            }
        }
    }
    assert!(skirt_moves > 0);
    if expected_roles.contains(&"Brim") {
        assert!(brim_moves > 0);
    } else {
        assert_eq!(brim_moves, 0);
    }
}
