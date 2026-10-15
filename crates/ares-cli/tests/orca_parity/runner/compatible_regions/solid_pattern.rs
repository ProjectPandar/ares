//! C2-S: local solid-pattern metadata follows PrintObject.cpp:3664-3709 /
//! Fill.cpp:891-893, separately from sparse-only anchoring at :1403-1406.
use crate::{self as parity, runner::OrcaRunner};
use serde_json::json;
use std::{fs, process::Command};

#[path = "solid_pattern_fixture.rs"]
mod fixture;
use fixture::Scope;

#[test]
fn local_object_grid_solid_matches_complete_native_output() {
    check(Scope::Object, false);
}
#[test]
fn local_model_part_grid_solid_matches_complete_native_output() {
    check(Scope::Part, false);
}
#[test]
fn local_grid_solid_without_bridge_candidates_matches_complete_native_output() {
    check(Scope::Object, true);
}

fn check(scope: Scope, low: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let mut control_solid = None;
    for pattern in ["rectilinear", "grid"] {
        let (exported, input_proof) = fixture::prepare(&runner, scope, pattern, low);
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        let directory = outcome.artifacts.as_ref().unwrap();
        let (bridge_moves, solid_moves) = observe(&sliced.reference);
        let solid_sha = fixture::hash(solid_moves.join("\n").as_bytes());
        let cli = Command::new(env!("CARGO_BIN_EXE_ares"))
            .arg("slice")
            .arg("-o")
            .arg(directory.join("cli.gcode"))
            .arg(directory.join("input.3mf"))
            .output()
            .unwrap();
        parity::artifacts::write(&directory.join("cli.stdout"), &cli.stdout).unwrap();
        parity::artifacts::write(&directory.join("cli.stderr"), &cli.stderr).unwrap();
        parity::artifacts::write(
            &directory.join("local-solid-witness.json"),
            &serde_json::to_vec_pretty(&json!({"input":input_proof,
                "native_internal_bridge_moves":bridge_moves,"native_solid_moves":solid_moves.len(),
                "native_solid_moves_sha256":solid_sha,"native_sha256":fixture::hash(&sliced.reference),
                "status":outcome.status,"detail":outcome.detail,"cli_exit":cli.status.code()}))
            .unwrap(),
        )
        .unwrap();
        assert!(
            std::str::from_utf8(&sliced.reference)
                .unwrap()
                .contains("; CONFIG_BLOCK_END")
        );
        assert_eq!(
            bridge_moves > 0,
            !low,
            "Native ceiling witness: {directory:?}"
        );
        assert!(
            !solid_moves.is_empty(),
            "Native solid role must be active: {directory:?}"
        );
        if pattern == "rectilinear" {
            control_solid = Some(solid_sha);
        } else {
            assert_ne!(
                control_solid.as_ref().unwrap(),
                &solid_sha,
                "Native local Grid must change solid moves"
            );
        }
        assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
        assert!(
            cli.status.success(),
            "CLI local solid pattern: {directory:?}"
        );
        let actual = fs::read(directory.join("ares.gcode")).unwrap();
        let cli_bytes = fs::read(directory.join("cli.gcode")).unwrap();
        assert_eq!(actual, cli_bytes, "API/CLI raw output: {directory:?}");
        parity::golden::compare_ordered_bytes_generator_only(&sliced.reference, &cli_bytes)
            .unwrap();
    }
}

fn observe(bytes: &[u8]) -> (usize, Vec<String>) {
    let mut role = "";
    let mut bridge_moves = 0;
    let mut solid_moves = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if line.starts_with("G1 ")
            && line.contains(" E")
            && (line.contains(" X") || line.contains(" Y"))
            && line
                .split_ascii_whitespace()
                .find_map(|token| token.strip_prefix('E'))
                .unwrap()
                .parse::<f64>()
                .unwrap()
                > 0.0
        {
            match role {
                "Internal Bridge" => bridge_moves += 1,
                "Internal solid infill" => solid_moves.push(line.to_owned()),
                _ => {}
            }
        }
    }
    (bridge_moves, solid_moves)
}

#[path = "bridge_anchor_max_projection.rs"]
mod anchor_max_projection;

#[path = "bridge_anchor_length_projection.rs"]
mod anchor_length_projection;

#[path = "zaa_contour_z.rs"]
mod zaa_contour_z;

#[path = "zaa_modifier_slice_z.rs"]
mod zaa_modifier_slice_z;

#[path = "zaa_fill_direction.rs"]
mod zaa_fill_direction;

#[path = "zaa_anchoring_direction.rs"]
mod zaa_anchoring_direction;

#[path = "zaa_locked_zag_direction.rs"]
mod zaa_locked_zag_direction;

#[path = "zaa_line_direction.rs"]
mod zaa_line_direction;

#[path = "zaa_honeycomb_direction.rs"]
mod zaa_honeycomb_direction;

#[path = "zaa_modifier_ironing_admission.rs"]
mod zaa_modifier_ironing_admission;
#[path = "zaa_object_ironing_direction.rs"]
mod zaa_object_ironing_direction;
