//! C2-T: local top-pattern metadata follows PrintObject.cpp:3664-3709 /
//! Fill.cpp:883-900, rather than the global preset's eight-pattern schema.
use crate::{self as parity, runner::OrcaRunner};
use serde_json::json;
use std::fs;

#[path = "top_pattern_fixture.rs"]
mod fixture;
#[path = "global_pattern.rs"]
mod global_pattern;
use fixture::Scope;

#[test]
fn local_object_grid_top_matches_complete_native_output() {
    check(Scope::Object, false);
}
#[test]
fn local_model_part_grid_top_matches_complete_native_output() {
    check(Scope::Part, false);
}
#[test]
fn local_grid_top_without_bridge_candidates_matches_complete_native_output() {
    check(Scope::Object, true);
}

fn check(scope: Scope, low: bool) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let mut control_top = None;
    for pattern in ["rectilinear", "grid"] {
        let (exported, input_proof) = fixture::prepare(&runner, scope, pattern, low);
        let sliced = runner.slice_case(&exported).unwrap();
        let outcome = parity::compare_case(&sliced);
        let directory = outcome.artifacts.as_ref().unwrap();
        let (bridge_moves, top_moves) = observe(&sliced.reference);
        let top_sha = fixture::hash(top_moves.join("\n").as_bytes());
        let cli = crate::runner::application::cli_command()
            .arg("slice")
            .arg("-o")
            .arg(directory.join("cli.gcode"))
            .arg(directory.join("input.3mf"))
            .output()
            .unwrap();
        parity::artifacts::write(&directory.join("cli.stdout"), &cli.stdout).unwrap();
        parity::artifacts::write(&directory.join("cli.stderr"), &cli.stderr).unwrap();
        parity::artifacts::write(
            &directory.join("local-top-witness.json"),
            &serde_json::to_vec_pretty(&json!({"input":input_proof,
                "native_internal_bridge_moves":bridge_moves,"native_top_moves":top_moves.len(),
                "native_top_moves_sha256":top_sha,"native_sha256":fixture::hash(&sliced.reference),
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
            !top_moves.is_empty(),
            "Native top role must be active: {directory:?}"
        );
        if pattern == "rectilinear" {
            control_top = Some(top_sha);
        } else {
            assert_ne!(
                control_top.as_ref().unwrap(),
                &top_sha,
                "Native local Grid must change top moves"
            );
        }
        assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
        assert!(cli.status.success(), "CLI local top pattern: {directory:?}");
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
    let mut top_moves = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if let Some(next) = line.strip_prefix(";TYPE:") {
            role = next;
        }
        if line.starts_with("G1 ")
            && line.contains(" E")
            && (line.contains(" X") || line.contains(" Y"))
        {
            match role {
                "Internal Bridge" => bridge_moves += 1,
                "Top surface" => top_moves.push(line.to_owned()),
                _ => {}
            }
        }
    }
    (bridge_moves, top_moves)
}
