use std::{collections::BTreeMap, process::Command};

use super::{FIXED_COMMIT, OptionInventoryRow, SourceCitation};

#[path = "provenance/axis.rs"]
mod axis;
#[path = "provenance/enums.rs"]
mod enums;

pub(super) fn verify_axis_defaults(source: &str, rows: &[OptionInventoryRow]) {
    axis::verify_axis_defaults(source, rows);
}

pub(super) fn verify_nozzle_type_default(
    print_config: &str,
    config: &str,
    common_defs: &str,
    rows: &[OptionInventoryRow],
) {
    enums::verify_nozzle_type_default(print_config, config, common_defs, rows);
}

pub(super) fn git_show(repo: &std::ffi::OsStr, path: &str) -> String {
    let output = Command::new("git")
        .arg("-C")
        .arg(repo)
        .args(["show", &format!("{FIXED_COMMIT}:{path}")])
        .output()
        .unwrap();
    assert!(output.status.success(), "git show failed for {path}");
    String::from_utf8(output.stdout).unwrap()
}

fn cited_line<'a>(sources: &'a BTreeMap<String, String>, citation: &SourceCitation) -> &'a str {
    let source = sources
        .get(&citation.path)
        .unwrap_or_else(|| panic!("unknown cited path {}", citation.path));
    source
        .lines()
        .nth(citation.line - 1)
        .unwrap_or_else(|| panic!("citation past EOF: {}:{}", citation.path, citation.line))
}

pub(super) fn verify_citation(sources: &BTreeMap<String, String>, citation: &SourceCitation) {
    let line = cited_line(sources, citation);
    assert!(
        line.contains(&citation.symbol),
        "missing cited symbol {} at {}:{}",
        citation.symbol,
        citation.path,
        citation.line
    );
}

pub(super) fn verify_consumer_citation(key: &str, is_metadata: bool, citation: &SourceCitation) {
    assert!(
        !matches!(
            citation.path.as_str(),
            "src/libslic3r/PrintConfig.hpp"
                | "src/libslic3r/PrintConfig.cpp"
                | "src/libslic3r/Preset.cpp"
        ),
        "{key} has a declaration/static-list consumer"
    );
    let generic_export = !is_metadata
        && citation.path == "src/libslic3r/GCode.cpp"
        && citation.symbol == "cfg.option(key)->is_nil()";
    let metadata_export = is_metadata
        && citation.path == "src/libslic3r/Format/bbs_3mf.cpp"
        && citation.symbol == "save_to_json";
    assert!(
        generic_export || metadata_export,
        "{key} lacks a runtime/export consumer at {}:{}",
        citation.path,
        citation.line
    );
}

pub(super) fn derive_export_rules(source: &str) -> BTreeMap<String, String> {
    let start = source
        .find("void GCode::append_full_config")
        .expect("missing append_full_config");
    let opening = source[start..].find('{').unwrap() + start;
    let body = balanced_block(source, opening);
    let mut rules = BTreeMap::new();

    let option_marker = r#"cfg.option<ConfigOptionFloats>("#;
    let multiplier_marker = "std::vector<double> temp_cfg_flush_multiplier = ";
    let multiplier_declaration = body
        .find(multiplier_marker)
        .expect("missing flush multiplier declaration")
        + multiplier_marker.len();
    assert!(body[multiplier_declaration..].starts_with(option_marker));
    let multiplier = quoted_at(body, multiplier_declaration + option_marker.len());
    let matrix_marker = "std::vector<double> temp_flush_volumes_matrix = ";
    let matrix_declaration = body
        .find(matrix_marker)
        .expect("missing flush matrix declaration")
        + matrix_marker.len();
    assert!(body[matrix_declaration..].starts_with(option_marker));
    let flush = quoted_at(body, matrix_declaration + option_marker.len());
    assert_ne!(flush, multiplier);
    let assignment_marker = ")->values = temp_flush_volumes_matrix";
    let assignment = body
        .find(assignment_marker)
        .expect("missing flush matrix assignment");
    let assignment_start = body[..assignment].rfind(option_marker).unwrap() + option_marker.len();
    assert_eq!(quoted_at(body, assignment_start), flush);
    assert!(body.contains("temp_cfg_flush_multiplier_idx = temp_cfg_flush_multiplier[idx]"));
    assert!(body.contains("std::round(inputx * temp_cfg_flush_multiplier_idx)"));
    rules.insert(flush.to_owned(), "scaled_flush_matrix".to_owned());

    assert!(body.contains("return banned_keys.find(key) != banned_keys.end();"));
    let loop_marker = "for (const std::string& key : cfg.keys())";
    let loop_start = body
        .find(loop_marker)
        .expect("missing config-key export loop");
    let loop_opening =
        body[loop_start + loop_marker.len()..].find('{').unwrap() + loop_start + loop_marker.len();
    let loop_body = balanced_block(body, loop_opening);
    let guard_marker = "if (!is_banned(key) && !cfg.option(key)->is_nil())";
    let guard_start = loop_body.find(guard_marker).expect("missing export guard");
    let guard_opening = loop_body[guard_start + guard_marker.len()..]
        .find('{')
        .unwrap()
        + guard_start
        + guard_marker.len();
    let guard_body = balanced_block(loop_body, guard_opening);
    let guard_closing = guard_opening + guard_body.len() + 1;
    assert!(loop_body[..guard_start].trim().is_empty());
    assert!(loop_body[guard_closing + 1..].trim().is_empty());

    let wipe_marker = r#"if (key == "#;
    let wipe_start = guard_body
        .find(wipe_marker)
        .expect("missing wipe tower branch")
        + wipe_marker.len();
    assert!(
        guard_body[..wipe_start - wipe_marker.len()]
            .trim()
            .is_empty()
    );
    let wipe_x = quoted_at(guard_body, wipe_start);
    let second_marker = r#" || key == "#;
    let second_start =
        guard_body[wipe_start..].find(second_marker).unwrap() + wipe_start + second_marker.len();
    let wipe_y = quoted_at(guard_body, second_start);
    let wipe_opening = guard_body[second_start..].find('{').unwrap() + second_start;
    let wipe_body = balanced_block(guard_body, wipe_opening);
    assert!(wipe_body.contains("std::setprecision(3)"));
    assert!(wipe_body.contains("get_at(print.get_plate_index())"));
    assert!(wipe_body.contains(r#"<< "; " << key << " = ""#));
    assert!(wipe_body.contains("cfg.option(key)"));
    for key in [wipe_x, wipe_y] {
        rules.insert(key.to_owned(), "plate_coordinate_duplicate".to_owned());
    }

    let wipe_closing = wipe_opening + wipe_body.len() + 1;
    let substitution_marker = r#"if(key == "#;
    let substitution_condition = guard_body[wipe_closing + 1..]
        .find(substitution_marker)
        .expect("missing colour substitution branch")
        + wipe_closing
        + 1;
    assert!(
        guard_body[wipe_closing + 1..substitution_condition]
            .trim()
            .is_empty()
    );
    let substitution_start = substitution_condition + substitution_marker.len();
    let target = quoted_at(guard_body, substitution_start);
    let source_marker = r#"cfg.opt_serialize("#;
    let source_start = guard_body[substitution_start..]
        .find(source_marker)
        .unwrap()
        + substitution_start
        + source_marker.len();
    let source = quoted_at(guard_body, source_start);
    assert_ne!(target, source);
    assert!(guard_body[substitution_start..].contains(r#"<< "; " << key << " = ""#));
    assert!(guard_body[substitution_start..].contains("else"));
    assert!(guard_body[substitution_start..].contains("cfg.opt_serialize(key)"));
    rules.insert(target.to_owned(), "filament_colour_substitution".to_owned());
    assert_eq!(rules.len(), 4);
    rules
}

pub(super) fn verify_rust_parser_mutations(source: &str) {
    let mutations = [
        (
            "std::vector<double> temp_flush_volumes_matrix = cfg.option<ConfigOptionFloats>(\"flush_volumes_matrix\")->values",
            "std::vector<double> temp_flush_volumes_matrix = cfg.option<ConfigOptionFloats>(\"flush_multiplier\")->values",
        ),
        (
            "cfg.option(key))->get_at",
            "cfg.option(\"flush_multiplier\"))->get_at",
        ),
        (
            "<< \"; \" << key << \" = \" << dynamic_cast",
            "<< \"; wrong = \" << dynamic_cast",
        ),
        (
            "cfg.opt_serialize(key)",
            "cfg.opt_serialize(\"flush_multiplier\")",
        ),
        (
            "if (!is_banned(key) && !cfg.option(key)->is_nil())",
            "if (!is_banned(key))",
        ),
        (
            "if (!is_banned(key) && !cfg.option(key)->is_nil())",
            "if (!cfg.option(key)->is_nil())",
        ),
        (
            "if(key == \"extruder_colour\")",
            "else if(key == \"extruder_colour\")",
        ),
        (
            "if (key == \"wipe_tower_x\" || key == \"wipe_tower_y\")",
            "ss << key; if (key == \"wipe_tower_x\" || key == \"wipe_tower_y\")",
        ),
    ];
    for (needle, replacement) in mutations {
        let mutated = replace_once(source, needle, replacement);
        assert!(
            std::panic::catch_unwind(|| derive_export_rules(&mutated)).is_err(),
            "Rust export parser accepted mutation {needle}"
        );
    }
}

fn replace_once(source: &str, needle: &str, replacement: &str) -> String {
    let first = source.find(needle).expect("missing mutation anchor");
    assert_eq!(
        source.rfind(needle),
        Some(first),
        "non-unique mutation anchor"
    );
    format!(
        "{}{}{}",
        &source[..first],
        replacement,
        &source[first + needle.len()..]
    )
}

fn quoted_at(source: &str, opening_quote: usize) -> &str {
    assert_eq!(source.as_bytes()[opening_quote], b'"');
    let value = &source[opening_quote + 1..];
    &value[..value.find('"').expect("unterminated quoted token")]
}

fn balanced_block(source: &str, opening: usize) -> &str {
    assert_eq!(source.as_bytes()[opening], b'{');
    let masked = mask_comments_and_strings(source);
    let mut depth = 1;
    for (offset, byte) in masked[opening + 1..].iter().enumerate() {
        match byte {
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    return &source[opening + 1..opening + 1 + offset];
                }
            }
            _ => {}
        }
    }
    panic!("unterminated append_full_config block")
}

fn mask_comments_and_strings(source: &str) -> Vec<u8> {
    let bytes = source.as_bytes();
    let mut masked = bytes.to_vec();
    let mut cursor = 0;
    while cursor < bytes.len() {
        let delimiter = match (bytes[cursor], bytes.get(cursor + 1)) {
            (b'/', Some(b'/')) => Some((b'\n', false)),
            (b'/', Some(b'*')) => Some((b'/', true)),
            (b'"', _) => Some((b'"', false)),
            (b'\'', _) => Some((b'\'', false)),
            _ => None,
        };
        let Some((ending, block_comment)) = delimiter else {
            cursor += 1;
            continue;
        };
        let start = cursor;
        cursor += if bytes[start] == b'/' { 2 } else { 1 };
        while cursor < bytes.len() {
            let terminates = if block_comment {
                bytes[cursor - 1] == b'*' && bytes[cursor] == ending
            } else {
                bytes[cursor] == ending && (ending == b'\n' || !is_escaped(bytes, cursor))
            };
            if terminates {
                cursor += usize::from(block_comment || ending != b'\n');
                break;
            }
            cursor += 1;
        }
        for byte in &mut masked[start..cursor] {
            if *byte != b'\n' {
                *byte = b' ';
            }
        }
    }
    masked
}

fn is_escaped(bytes: &[u8], index: usize) -> bool {
    bytes[..index]
        .iter()
        .rev()
        .take_while(|&&byte| byte == b'\\')
        .count()
        % 2
        == 1
}

pub(super) fn reconstruct_inventory(repo: &std::ffi::OsStr) -> Vec<OptionInventoryRow> {
    let output = generator(repo, "--stdout");
    assert!(
        output.status.success(),
        "fixed-source inventory reconstruction failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).unwrap()
}

pub(super) fn verify_source_mutations(repo: &std::ffi::OsStr) {
    let output = generator(repo, "--verify-mutations");
    assert!(
        output.status.success(),
        "source-semantics mutation verification failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert_eq!(
        String::from_utf8(output.stdout).unwrap(),
        "verified 19 source-semantics mutations\n"
    );
}

fn generator(repo: &std::ffi::OsStr, argument: &str) -> std::process::Output {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(std::path::Path::parent)
        .unwrap();
    Command::new("node")
        .arg("scripts/generate_task5_inventory.mjs")
        .arg(argument)
        .env("ORCA_SLICER_REPO", repo)
        .current_dir(root)
        .output()
        .unwrap()
}
