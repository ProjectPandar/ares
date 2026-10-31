//! `GCode.cpp:5808-5813,6515-6526`: default scarf speed must not reselect small loops.

use super::{ExportedCase, OrcaRunner, cache_identity};
use std::fs;

fn check(name: &str, input_bytes: &[u8], input_sha256: &str, expected_feedrates: &[f64]) {
    assert_eq!(cache_identity::raw_digest(input_bytes), input_sha256);
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("scarf-default-threshold-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    let output = directory.join("native");
    fs::write(&input, input_bytes).unwrap();
    fs::create_dir(&output).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: format!("scarf-default-speed/{name}"),
            project: input_bytes.to_vec(),
            project_path: input,
            output_dir: output,
        })
        .unwrap();
    let outcome = crate::compare_case(&sliced);
    let artifacts = outcome.artifacts.as_ref().unwrap();
    let actual = fs::read(artifacts.join("ares.gcode")).unwrap();
    let native_feedrates = outer_feedrates(&sliced.reference);
    let actual_feedrates = outer_feedrates(&actual);
    fs::write(
        directory.join("speed-witness.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "input_sha256": input_sha256,
            "native_sha256": cache_identity::raw_digest(&sliced.reference),
            "ares_sha256": cache_identity::raw_digest(&actual),
            "native_outer_nonfirst_feedrates": native_feedrates,
            "ares_outer_nonfirst_feedrates": actual_feedrates,
            "status": outcome.status,
            "detail": outcome.detail,
            "artifacts": artifacts,
            "comparison": "complete bytes; generated-by identity only; M73/footer retained"
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(native_feedrates, expected_feedrates);
    assert!(
        std::str::from_utf8(&sliced.reference)
            .unwrap()
            .contains("M73 P")
    );
    assert_eq!(outcome.status, "PASS", "{}; {directory:?}", outcome.detail);
    assert_eq!(actual_feedrates, native_feedrates);
}

#[test]
fn scarf_default_threshold_matches_complete_output() {
    check(
        "scaled-mm-disagreement",
        include_bytes!("../../../../../tests/parity/scarf-default-threshold/boundary.3mf"),
        "1646ba992cc58685074abd95e009d17f85f88e190379912201d9d9f88399f4e5",
        &[3600.0, 7200.0],
    );
}

#[test]
fn scarf_default_percentage_matches_complete_output() {
    check(
        "percentage",
        include_bytes!("../../../../../tests/parity/scarf-default-threshold/percentage.3mf"),
        "88da8bb4273865225bf38359379a14d1947c83cd71e51a05076d5c4cfcc78782",
        &[3600.0, 7200.0],
    );
}

#[test]
fn scarf_default_absolute_matches_complete_output() {
    check(
        "absolute",
        include_bytes!("../../../../../tests/parity/scarf-default-threshold/absolute.3mf"),
        "2e982da9e5eb5cadc889996a2579196027c8baa4b47bcc8cd414579f81954ba6",
        &[3600.0, 7200.0],
    );
}

#[test]
fn scarf_default_above_role_matches_complete_output() {
    check(
        "above-role",
        include_bytes!("../../../../../tests/parity/scarf-default-threshold/above-role.3mf"),
        "3884f95ff09b00210eab4c06c772a16b5dfa64ae231a766a0a42301132e75366",
        &[7200.0],
    );
}

fn outer_feedrates(bytes: &[u8]) -> Vec<f64> {
    let mut layer = 0;
    let mut role = "";
    let mut current = 0.0;
    let mut result = Vec::new();
    for line in std::str::from_utf8(bytes).unwrap().lines() {
        if line == ";LAYER_CHANGE" {
            layer += 1;
        }
        if let Some(value) = line.strip_prefix(";TYPE:") {
            role = value;
        }
        if line.starts_with("G1 ") {
            let words = line.split_whitespace().collect::<Vec<_>>();
            if let Some(value) = words.iter().find_map(|word| word.strip_prefix('F')) {
                current = value.parse().unwrap();
            }
            if layer > 1
                && role == "Outer wall"
                && words
                    .iter()
                    .any(|word| word.starts_with('X') || word.starts_with('Y'))
                && words.iter().any(|word| {
                    word.strip_prefix('E')
                        .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
                })
            {
                result.push(current);
            }
        }
    }
    result.sort_by(f64::total_cmp);
    result.dedup();
    result
}
