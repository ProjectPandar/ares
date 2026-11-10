//! Pre-heating and pre-cooling injector on a two-extruder print
//! (`docs/architecture/pre-heating-injector-spec.md`).

use super::fixture::{Overrides, configured_project};
use serde_json::json;
use std::fs;

/// One injection: the 1-based output line number and text of the line before it, then the
/// injector lines.
type Injection = (usize, String, Vec<String>);

fn injector_m104(line: &str, kind: &str) -> bool {
    line.starts_with("M104 ") && line.contains(&format!(" N0 ;Multi extruder pre {kind}"))
}

/// The length of the injector group starting at `lines[0]`: an `M632`/`M400`/`M104`/`M633`
/// heating group or an `M400`/`M104` cooling pair.
fn group(lines: &[&str]) -> usize {
    match lines {
        [m632, "M400", m104, "M633", ..]
            if m632.starts_with("M632 ") && injector_m104(m104, "heating") =>
        {
            4
        }
        ["M400", m104, ..] if injector_m104(m104, "cooling") => 2,
        _ => 0,
    }
}

fn injections(gcode: &[u8]) -> Vec<Injection> {
    let text = std::str::from_utf8(gcode).unwrap();
    let lines = text.lines().collect::<Vec<_>>();
    let mut injections = Vec::new();
    let mut index = 1;
    while index < lines.len() {
        let mut end = index;
        while end < lines.len() {
            let length = group(&lines[end..]);
            if length == 0 {
                break;
            }
            end += length;
        }
        if end == index {
            index += 1;
            continue;
        }
        injections.push((
            index,
            lines[index - 1].to_owned(),
            lines[index..end]
                .iter()
                .map(|line| (*line).to_owned())
                .collect(),
        ));
        index = end;
    }
    injections
}

#[test]
fn injected_temperature_lines_and_positions_match_native() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let (project, reference, actual) = configured_project(
        "",
        "",
        false,
        Overrides {
            machine: &[("enable_pre_heating", json!("1"))],
            ..Default::default()
        },
    );
    let native = std::str::from_utf8(&reference).unwrap();
    assert!(
        native
            .lines()
            .any(|line| line == "; MACHINE_START_GCODE_END"),
        "Native must mark the end of the machine start G-code"
    );
    for kind in ["cooling", "heating"] {
        assert!(
            native.lines().any(|line| injector_m104(line, kind)),
            "Native must inject pre {kind}"
        );
    }
    for tool in ["T0", "T1"] {
        assert!(
            native.lines().any(|line| line == tool),
            "Native must select `{tool}`"
        );
    }

    let root = crate::artifacts::root_from_env().unwrap();
    let directory = tempfile::Builder::new()
        .prefix("pre-heating-")
        .tempdir_in(&root)
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), &project).unwrap();
    let cli = crate::runner::application::cli_command()
        .arg("slice")
        .arg("-o")
        .arg(directory.join("cli.gcode"))
        .arg(directory.join("input.3mf"))
        .output()
        .unwrap();
    assert!(cli.status.success(), "CLI slice: {directory:?}");
    assert_eq!(
        actual,
        fs::read(directory.join("cli.gcode")).unwrap(),
        "API/CLI raw output: {directory:?}"
    );

    assert_eq!(
        injections(&actual),
        injections(&reference),
        "injected lines and the lines they follow: {directory:?}"
    );
}
