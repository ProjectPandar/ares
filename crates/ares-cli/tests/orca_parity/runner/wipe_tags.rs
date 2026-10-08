//! Native GCodeProcessor::process_tags WIPE replay (cpp:3146-3155).
//! Failure inventory before GREEN: BBL-spaced tags accepted on non-BBL,
//! ignored END resetting state, compatible prefix suffix rejected, ordinary
//! extrusion lost from role accounting, or canonical BBL behavior changed.

use super::{ExportedCase, OrcaRunner};
use ares_core::gcode_processor::EMoveType;

const PROJECT: &[u8] =
    include_bytes!("../../../../../tests/parity/validation/kobra3-centered/0.8.3mf");

fn replay(name: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project_path = runner.work.join("input.3mf");
    std::fs::write(&project_path, PROJECT).unwrap();
    let output_dir = runner.work.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "processor/compatible-wipe-tags".into(),
            project: PROJECT.to_vec(),
            project_path,
            output_dir,
        })
        .unwrap();
    let source = std::str::from_utf8(&case.reference).unwrap();
    let (input, selected_line) = variant(source, name);
    let input_path = runner.work.join(format!("{name}.gcode"));
    std::fs::write(&input_path, &input).unwrap();
    let sha = super::cache_identity::raw_digest(input.as_bytes());
    let Some(oracle) = runner.processor_reference(&input_path).unwrap() else {
        return;
    };
    let native: serde_json::Value =
        serde_json::from_slice(&std::fs::read(oracle.join("role-map.json")).unwrap()).unwrap();
    let bbl = name == "bbl-canonical";
    assert_eq!(native["input_sha256"], sha);
    assert_eq!(native["bbl_printer"], bbl);
    let project = ares_core::load_project(PROJECT).unwrap();
    let processed =
        ares_core::gcode_processor::process_gcode(&input, project.settings(), bbl, false);
    let moves = processed.result.moves;
    let statistics = serde_json::to_value(processed.result.print_statistics).unwrap();
    let artifacts = runner.work.join("compatible-wipe-tags").join(name);
    std::fs::create_dir_all(&artifacts).unwrap();
    std::fs::write(
        artifacts.join("ares-statistics.json"),
        serde_json::to_vec_pretty(&statistics).unwrap(),
    )
    .unwrap();
    std::fs::write(artifacts.join("ares-moves.txt"), format!("{moves:#?}")).unwrap();
    std::fs::write(
        artifacts.join("input-receipt.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "project_sha256":super::cache_identity::raw_digest(PROJECT),
            "native_source_sha256":super::cache_identity::raw_digest(&case.reference),
            "variant":name,"input_sha256":sha,"selected_line":selected_line,
        }))
        .unwrap(),
    )
    .unwrap();
    let roles = statistics["used_filaments_per_role"].as_object().unwrap();
    let rows = native["roles"].as_array().unwrap();
    assert_eq!(
        roles.len(),
        rows.len(),
        "{name}: complete Native role key set"
    );
    for row in rows {
        let role = row["role"].as_str().unwrap();
        let pair = roles[role].as_array().unwrap();
        assert_eq!(pair.len(), 2);
        for (index, value) in pair.iter().enumerate() {
            assert_eq!(
                value.as_f64().unwrap().to_bits(),
                row["bits"][index].as_u64().unwrap(),
                "{name}: {role}, pair {index} (meters, grams)"
            );
        }
    }
    let csv = std::fs::read_to_string(oracle.join("moves.csv")).unwrap();
    let source_id = selected_line.to_string();
    let native_move = csv
        .lines()
        .skip(1)
        .map(|line| line.split(',').collect::<Vec<_>>())
        .find(|fields| fields[0] == source_id)
        .unwrap();
    let expected = match native_move[1] {
        "8" => EMoveType::Travel,
        "9" => EMoveType::Wipe,
        "10" => EMoveType::Extrude,
        other => panic!("unexpected Native type at fixture line {selected_line}: {other}"),
    };
    let actual = moves
        .iter()
        .find(|vertex| vertex.gcode_id as usize == selected_line)
        .unwrap();
    assert_eq!(
        actual.move_type, expected,
        "{name}: Native source line {selected_line}"
    );
}

// Only comment tags/configured producer dialect change; retain every motion
// command from the runtime slice. Line count is preserved for Native CSV IDs.
fn variant(source: &str, name: &str) -> (String, usize) {
    let lines: Vec<_> = source.lines().collect();
    let start = lines
        .iter()
        .position(|line| *line == ";WIPE_START")
        .unwrap();
    let end = start
        + lines[start..]
            .iter()
            .position(|line| *line == ";WIPE_END")
            .unwrap();
    let motion_start = if name == "non-bbl-spaced-end" {
        end + 1
    } else {
        start + 1
    };
    let selected = motion_start
        + lines[motion_start..]
            .iter()
            .position(|line| line.starts_with("G1 "))
            .unwrap()
        + 1;
    let input = lines
        .into_iter()
        .map(|line| match (name, line) {
            ("non-bbl-spaced" | "bbl-canonical", ";WIPE_START") => "; WIPE_START".into(),
            ("non-bbl-spaced" | "non-bbl-spaced-end" | "bbl-canonical", ";WIPE_END") => {
                "; WIPE_END".into()
            }
            ("non-bbl-prefix", ";WIPE_START") => ";WIPE_START suffix".into(),
            ("non-bbl-prefix", ";WIPE_END") => ";WIPE_END suffix".into(),
            ("bbl-canonical", line) if line.starts_with(";TYPE:") => {
                line.replacen(";TYPE:", "; FEATURE: ", 1)
            }
            ("bbl-canonical", line) if line.starts_with("; printer_model = ") => {
                "; printer_model = Bambu Lab X1 Carbon".into()
            }
            _ => line.to_owned(),
        })
        .collect::<Vec<_>>()
        .join("\n")
        + "\n";
    (input, selected)
}

#[test]
fn non_bbl_spaced_wipe_tags_match_native() {
    replay("non-bbl-spaced");
}
#[test]
fn non_bbl_compatible_wipe_control_matches_native() {
    replay("non-bbl-compatible");
}
#[test]
fn non_bbl_spaced_end_does_not_close_native_wipe() {
    replay("non-bbl-spaced-end");
}
#[test]
fn non_bbl_wipe_prefix_matches_native() {
    replay("non-bbl-prefix");
}
#[test]
fn bbl_canonical_wipe_control_matches_native() {
    replay("bbl-canonical");
}
