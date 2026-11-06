//! Runtime Native travel references through the original public CLI boundary.
//! AvoidCrossingPerimeters.cpp:174-294,1233-1288 and GCode.cpp:6013-6032,
//! 6150-6175,7350-7355 own the routing, writer-only inward move and logical
//! cursor semantics. Five original tests retain ordered XY/full-stream/layer
//! assertions; only the validated generator/date line is normalized.
//! OrcaRunner owns required/optional dependencies and slice_case caching.
//! See native-artifact-cleanup-travel-reference-spec.md.

use super::{ExportedCase, OrcaRunner, cache_identity, repo_root};
use std::fs;

#[test]
fn avoid_crossing_perimeters_preserves_ordered_xy_travel_commands() {
    let Some((actual, expected)) = slice_fixture("anchor") else {
        return;
    };
    let actual = xy_travels(&actual);
    let expected = xy_travels(&expected);
    assert_eq!(actual.len(), expected.len(), "XY travel command count");
    for (index, (actual, expected)) in actual.iter().zip(&expected).enumerate() {
        assert_eq!(actual, expected, "XY travel command {index}");
    }
}

#[test]
fn closest_line_extension_preserves_all_ordered_one_wall_xy_commands() {
    let Some((actual, expected)) = slice_fixture("one-wall") else {
        return;
    };
    let xy = |gcode: &str| {
        gcode
            .lines()
            .filter(|line| line.starts_with("G1 X") && !line.contains(" E"))
            .map(str::to_owned)
            .collect::<Vec<_>>()
    };
    let expected = xy(&expected);
    assert_eq!(expected.len(), 436);
    assert_eq!(
        xy(&actual),
        expected,
        "all ordered XY, including repeated corners"
    );
}

#[test]
fn closest_line_extension_matches_complete_one_wall_orca_artifact() {
    let Some((actual, expected)) = slice_fixture("one-wall") else {
        return;
    };
    assert_eq!(
        normalize_generator(&actual, "Ares"),
        normalize_generator(&expected, "OrcaSlicer"),
        "complete one-wall G-code, including whitespace, statistics and M73"
    );
}

#[test]
fn extrude_loop_preserves_complete_ordered_sparse_runs_and_inward_move() {
    let Some((actual, expected)) = slice_fixture("anchor") else {
        return;
    };
    for z in ["2.58", "2.72", "8.04"] {
        let actual = layer(&actual, z);
        let expected = layer(&expected, z);
        assert!(expected.contains("G1 X164.851 Y179.88\n"));
        assert!(expected.contains(";TYPE:Sparse infill\n"));
        // Compare the entire layer, including the inward move, retraction,
        // approach, both sparse paths, intervening travel, feeds and M73.
        assert_eq!(actual, expected, "complete layer at Z{z}");
    }
}

#[test]
fn two_wall_inward_move_preserves_orca_layer_sections() {
    let Some((actual, expected)) = slice_fixture("two-walls") else {
        return;
    };
    for z in ["0.34", "0.76", "1.18"] {
        assert_eq!(
            layer(&actual, z),
            layer(&expected, z),
            "complete layer at Z{z}"
        );
    }
}

fn slice_fixture(name: &str) -> Option<(String, String)> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let source = repo_root()
        .join("tests/avoid_crossing_travel")
        .join(format!("{name}.3mf"));
    let project = fs::read(&source).unwrap();
    let project_path = runner.work.join(source.file_name().unwrap());
    fs::write(&project_path, &project).unwrap();
    let output_dir = runner.work.join("native");
    fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: format!("avoid-crossing-travel/{name}"),
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let output = runner.work.join("ares.gcode");
    let binary =
        std::env::var_os("ARES_TRAVEL_BIN").unwrap_or_else(|| env!("CARGO_BIN_EXE_ares").into());
    let result = super::application::CliCommand::new(&binary)
        .arg("slice")
        .arg("-o")
        .arg(&output)
        .arg(&source)
        .output()
        .unwrap();
    fs::write(runner.work.join("ares.stdout"), &result.stdout).unwrap();
    fs::write(runner.work.join("ares.stderr"), &result.stderr).unwrap();
    assert!(result.status.success(), "{result:?}");
    let actual = fs::read_to_string(output).unwrap();
    fs::write(runner.work.join("orca.gcode"), &case.reference).unwrap();
    fs::write(
        runner.work.join("travel-reference-comparison.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "fixture": name,
            "input_sha256": cache_identity::raw_digest(&case.project),
            "ares_sha256": cache_identity::raw_digest(actual.as_bytes()),
            "native_sha256": cache_identity::raw_digest(&case.reference),
            "ares_binary": binary,
            "ares_exit": result.status.code(),
            "artifacts": runner.work,
        }))
        .unwrap(),
    )
    .unwrap();
    Some((actual, String::from_utf8(case.reference).unwrap()))
}

fn xy_travels(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| line.starts_with("G1 X") && !line.contains(" E"))
        .collect()
}

fn normalize_generator(gcode: &str, producer: &str) -> String {
    let expression = if producer == "Ares" {
        r"(?m)^; generated by Ares 2\.5\.0-beta$".to_owned()
    } else {
        format!(
            r"(?m)^; generated by {producer} 2\.5\.0-beta on (\d{{4}}-\d{{2}}-\d{{2}}) at (\d{{2}}:\d{{2}}:\d{{2}})$"
        )
    };
    let pattern = regex::Regex::new(&expression).unwrap();
    let captures = pattern.captures_iter(gcode).collect::<Vec<_>>();
    assert_eq!(captures.len(), 1, "one valid {producer} generator line");
    if producer != "Ares" {
        format!("{}T{}", &captures[0][1], &captures[0][2])
            .parse::<jiff::civil::DateTime>()
            .unwrap();
    }
    pattern
        .replace(gcode, "; generated by <GENERATOR>")
        .into_owned()
}

fn layer<'a>(gcode: &'a str, z: &str) -> &'a str {
    let marker = format!(";Z:{z}\n");
    let start = gcode.find(&marker).unwrap();
    let rest = &gcode[start..];
    &rest[..rest.find(";LAYER_CHANGE\n").unwrap()]
}
