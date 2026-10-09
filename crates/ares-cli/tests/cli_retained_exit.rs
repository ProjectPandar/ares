//! OrcaSlicer.cpp:6209-6227 exports before CLI_GCODE_PATH_IN_UNPRINTABLE_AREA.
//! Fresh seed0/cpu1 capture: /tmp/worker4-102-native-xPSj5F; worker4 RED.
#![cfg(target_os = "linux")]

use std::{fs, path::Path, process::Command};

#[test]
fn retained_reference_requires_native_not_frozen_outputs() {
    let output = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "cli_unprintable_area_retains_complete_native_gcode_before_exit_154",
            "--nocapture",
        ])
        .env("ARES_REQUIRE_ORCA", "1")
        .env_remove("ARES_ORCA_NATIVE_BIN")
        .output()
        .unwrap();
    let diagnostic = format!(
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(diagnostic.contains("running 1 test"), "selector drift: {diagnostic}");
    assert!(
        !output.status.success() && diagnostic.contains("ARES_ORCA_NATIVE_BIN"),
        "frozen retained reference hid required Native absence: {diagnostic}"
    );
}

#[test]
fn cli_unprintable_area_retains_complete_native_gcode_before_exit_154() {
    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/validation");
    let expected = fixtures.join("cli-exit");
    let temp = tempfile::tempdir().unwrap();
    let retained = temp.path().join("plate_1.gcode");
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "-o"])
        .arg(&retained)
        .arg(fixtures.join("o6-stock-circle-inward.3mf"))
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(154), "typed -102 exit");
    assert_eq!(
        output.stdout,
        fs::read(expected.join("path-outside.stdout")).unwrap()
    );
    assert_eq!(
        output.stderr,
        fs::read(expected.join("path-outside.stderr")).unwrap()
    );
    assert_eq!(
        fs::read(temp.path().join("result.json")).unwrap(),
        fs::read(expected.join("path-outside.result.json")).unwrap(),
        "complete native result.json, including applied configuration"
    );
    let actual = fs::read_to_string(&retained).unwrap();
    let native = fs::read_to_string(expected.join("path-outside.native.gcode")).unwrap();
    assert_eq!(
        normalize_generator(&actual, "Ares"),
        normalize_generator(&native, "OrcaSlicer"),
        "retained complete stream: generator-only normalization"
    );
}

#[test]
fn cli_unprintable_area_reports_selected_nonminimum_plate() {
    use std::io::{Cursor, Read, Write};

    let fixtures = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/parity/validation");
    let temp = tempfile::Builder::new()
        .prefix("cli-retained-selected-plate-")
        .tempdir()
        .unwrap()
        .keep();
    let input = temp.join("selected-plate.3mf");
    let original = fs::read(fixtures.join("o6-stock-circle-inward.3mf")).unwrap();
    let mut archive = zip::ZipArchive::new(Cursor::new(original)).unwrap();
    let mut writer = zip::ZipWriter::new(fs::File::create(&input).unwrap());
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let xml = String::from_utf8(bytes).unwrap();
            assert_eq!(xml.matches("<plate>").count(), 1);
            assert_eq!(xml.matches("key=\"plater_id\" value=\"1\"").count(), 1);
            bytes = xml
                .replace("key=\"plater_id\" value=\"1\"", "key=\"plater_id\" value=\"2\"")
                .replace(
                    "  <assemble>",
                    "  <plate>\n    <metadata key=\"plater_id\" value=\"1\"/>\n  </plate>\n  <assemble>",
                )
                .into_bytes();
        } else if entry.name() == "3D/3dmodel.model" {
            // PartPlate::compute_origin: second plate's X origin is 110 * 1.2.
            let xml = String::from_utf8(bytes).unwrap();
            let transform = "transform=\"1 0 0 0 1 0 0 0 1 -0.1 0 0\"";
            assert_eq!(xml.matches(transform).count(), 1);
            bytes = xml
                .replace(transform, "transform=\"1 0 0 0 1 0 0 0 1 131.9 0 0\"")
                .into_bytes();
        } else if entry.name() == "Metadata/filament_sequence.json" {
            bytes = br#"{"plate_2":{"nozzle_sequence":[],"optimal_assignment":[],"sequence":[]},"plate_1":{"nozzle_sequence":[],"optimal_assignment":[],"sequence":[]}}"#.to_vec();
        }
        writer
            .start_file(
                entry.name(),
                zip::write::SimpleFileOptions::default().compression_method(entry.compression()),
            )
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap();

    let retained = temp.join("plate_2.gcode");
    let output = Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--plate", "2", "-o"])
        .arg(&retained)
        .arg(&input)
        .output()
        .unwrap();
    assert_eq!(output.status.code(), Some(154), "{temp:?}: {output:?}");
    assert_eq!(
        output.stdout,
        fs::read(fixtures.join("cli-exit/path-outside.stdout")).unwrap()
    );
    assert_eq!(
        output.stderr,
        fs::read(fixtures.join("cli-exit/path-outside.stderr")).unwrap()
    );
    let expected = fs::read_to_string(fixtures.join("cli-exit/path-outside.result.json"))
        .unwrap()
        .replace("\"plate_index\": 1", "\"plate_index\": 2");
    assert_eq!(
        fs::read_to_string(temp.join("result.json")).unwrap(),
        expected
    );
    assert!(!fs::read(retained).unwrap().is_empty());
}

fn normalize_generator(gcode: &str, producer: &str) -> String {
    let expression = if producer == "Ares" {
        r"(?m)^; generated by Ares 2\.4\.2$".to_owned()
    } else {
        format!(
            r"(?m)^; generated by {producer} 2\.4\.2 on (\d{{4}}-\d{{2}}-\d{{2}}) at (\d{{2}}:\d{{2}}:\d{{2}})$"
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
