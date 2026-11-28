use std::{
    fs,
    io::{Cursor, Read, Write},
    process::{Command, Output},
};

use serde_json::{Value, json};

use crate::{OrcaRunner, golden};

const CUBE: &[u8] = include_bytes!("../../../../tests/parity/mesh-edge-pairing/cube-control.3mf");
const START: &str = "; imex-start={imex_mode}:{imex_mode_index}:{imex_mode_gcode}";

fn project(is_imex: bool, script: &str, scripts: (&str, &str, &str)) -> Vec<u8> {
    project_with_settings(is_imex, script, scripts, |settings| {
        serde_json::to_vec(settings).unwrap()
    })
}

fn project_with_settings(
    is_imex: bool,
    script: &str,
    (header, start, end): (&str, &str, &str),
    encode: impl Fn(&mut Value) -> Vec<u8>,
) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(CUBE)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            settings["is_imex"] = json!(if is_imex { "1" } else { "0" });
            settings["imex_parallel_mode"] = json!("primary");
            settings["imex_gantry_count"] = json!("2");
            settings["imex_tools_per_gantry"] = json!("1");
            settings["imex_mode_names"] = json!(["unused", "primary"]);
            settings["imex_mode_active_tools"] = json!(["0:P,1:C", "0:P"]);
            settings["imex_mode_gcodes"] = json!(["; imex-wrong-row", script]);
            settings["file_start_gcode"] = json!(header);
            settings["machine_start_gcode"] = json!(start);
            settings["machine_end_gcode"] = json!(end);
            settings["before_layer_change_gcode"] = json!("G92 E0");
            bytes = encode(&mut settings);
        }
        output
            .start_file(file.name(), zip::write::SimpleFileOptions::DEFAULT)
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn slice_pair(
    label: &str,
    input: &[u8],
    expected_exit: i32,
) -> Option<[(Output, Option<String>); 2]> {
    let runner = OrcaRunner::from_env().unwrap()?;
    let directory = tempfile::Builder::new()
        .prefix(label)
        .tempdir_in(runner.work_dir())
        .unwrap()
        .keep();
    fs::write(directory.join("input.3mf"), input).unwrap();
    let out = directory.join("out");
    fs::create_dir(&out).unwrap();
    let args = ["--slice", "1", "--outputdir", "out", "input.3mf"];
    let result = [
        ("native", Command::new(runner.bin())),
        ("ares", Command::new(env!("CARGO_BIN_EXE_ares"))),
    ]
    .map(|(side, mut command)| {
        let output = command.current_dir(&directory).args(args).output().unwrap();
        fs::write(directory.join(format!("{side}.stdout")), &output.stdout).unwrap();
        fs::write(directory.join(format!("{side}.stderr")), &output.stderr).unwrap();
        fs::write(
            directory.join(format!("{side}.exit")),
            format!("{:?}\n", output.status.code()),
        )
        .unwrap();
        let gcode = out.join("plate_1.gcode");
        let gcode = gcode.exists().then(|| {
            let path = directory.join(format!("{side}.gcode"));
            fs::rename(gcode, &path).unwrap();
            fs::read_to_string(path).unwrap()
        });
        let result = out.join("result.json");
        if result.exists() {
            fs::rename(result, directory.join(format!("{side}.result.json"))).unwrap();
        }
        (output, gcode)
    });
    for (output, _) in &result {
        assert_eq!(output.status.code(), Some(expected_exit), "{directory:?}");
    }
    Some(result)
}

fn markers(gcode: &str) -> Vec<&str> {
    gcode
        .lines()
        .filter(|line| line.starts_with("; imex-"))
        .collect()
}

#[test]
fn imex_mode_script_shares_globals_with_file_and_machine_start() {
    let input = project(
        true,
        "{global mode_marker = 7}; imex-script={mode_marker}",
        (
            "; imex-header={mode_marker}",
            "; imex-start={imex_mode}:{imex_mode_index}:{mode_marker}",
            "; imex-end={imex_mode}:{imex_mode_index}:{mode_marker}",
        ),
    );
    let Some([(_, native), (_, ares)]) = slice_pair("imex-mode-script", &input, 0) else {
        return;
    };
    for output in [native.unwrap(), ares.unwrap()] {
        assert_eq!(
            markers(&output),
            [
                "; imex-header=7",
                "; imex-script=7",
                "; imex-start=primary:1:7",
                "; imex-end=primary:1:7",
            ]
        );
        assert!(output.contains(";TYPE:Custom\n; imex-script=7\n; imex-start=primary:1:7\n"));
    }
}

#[test]
fn empty_imex_script_keeps_the_selected_mode_placeholders() {
    let input = project(true, "", ("", START, ""));
    let Some([(_, native), (_, ares)]) = slice_pair("imex-empty-script", &input, 0) else {
        return;
    };
    for output in [native.unwrap(), ares.unwrap()] {
        assert_eq!(markers(&output), ["; imex-start=primary:1:"]);
        assert!(output.contains(";TYPE:Custom\n; imex-start=primary:1:\n"));
    }
}

#[test]
fn non_imex_export_does_not_render_a_populated_mode_table() {
    let input = project(false, "; imex-disabled-script", ("", START, ""));
    let Some([(_, native), (_, ares)]) = slice_pair("imex-disabled", &input, 0) else {
        return;
    };
    for output in [native.unwrap(), ares.unwrap()] {
        assert_eq!(markers(&output), ["; imex-start=:0:"]);
    }
}

#[test]
fn legacy_ixex_project_settings_render_complete_imex_startup() {
    let input = project_with_settings(true, "; imex-script=legacy", ("", START, ""), |settings| {
        let fields = settings.as_object_mut().unwrap();
        for (modern, legacy) in [
            ("is_imex", "is_ixex"),
            ("imex_parallel_mode", "ixex_parallel_mode"),
            ("imex_gantry_count", "ixex_gantry_count"),
            ("imex_tools_per_gantry", "ixex_tools_per_gantry"),
            ("imex_mode_names", "ixex_mode_names"),
            ("imex_mode_active_tools", "ixex_mode_active_tools"),
            ("imex_mode_gcodes", "ixex_mode_gcodes"),
        ] {
            let value = fields.remove(modern).unwrap();
            fields.insert(legacy.to_owned(), value);
        }
        serde_json::to_vec(settings).unwrap()
    });
    let Some([(_, native), (_, ares)]) = slice_pair("imex-legacy-startup-", &input, 0) else {
        return;
    };
    let native = native.unwrap();
    let ares = ares.unwrap();
    assert_eq!(
        markers(&native),
        [
            "; imex-script=legacy",
            "; imex-start=primary:1:; imex-script=legacy"
        ]
    );
    golden::compare_ordered_bytes_generator_only(native.as_bytes(), ares.as_bytes()).unwrap();
}

#[test]
fn legacy_ixex_project_keys_override_canonical_keys_in_saved_settings() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = project_with_settings(true, "", ("", START, ""), |settings| {
        let fields = settings.as_object_mut().unwrap();
        let enabled = fields.remove("is_imex").unwrap();
        fields.insert("is_ixex".into(), enabled);
        fields.insert("ixex_carriage_width_x".into(), json!("7"));
        fields.insert("ixex_carriage_width_y".into(), json!("11"));
        let mut wire = serde_json::to_vec(settings).unwrap();
        // File order conflicts with sorted key order. Do not reserialize through a map.
        assert_eq!(wire.pop(), Some(b'}'));
        wire.extend_from_slice(
            br#", "is_imex":"0", "imex_nozzle_clearance_x":"41", "imex_nozzle_clearance_y":"43"}"#,
        );
        wire
    });
    let root = tempfile::Builder::new()
        .prefix("imex-legacy-saved-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    fs::write(root.join("input.3mf"), input).unwrap();
    let export = |side, bin: &std::path::Path| {
        let filename = format!("{side}.3mf");
        let output = Command::new(bin)
            .current_dir(&root)
            .args([
                "--arrange",
                "0",
                "--orient",
                "0",
                "--export-3mf",
                &filename,
                "input.3mf",
            ])
            .output()
            .unwrap();
        fs::write(root.join(format!("{side}.stdout")), &output.stdout).unwrap();
        fs::write(root.join(format!("{side}.stderr")), &output.stderr).unwrap();
        assert!(output.status.success(), "{side} export: {root:?}");
        let mut archive =
            zip::ZipArchive::new(fs::File::open(root.join(filename)).unwrap()).unwrap();
        let mut bytes = Vec::new();
        archive
            .by_name("Metadata/project_settings.config")
            .unwrap()
            .read_to_end(&mut bytes)
            .unwrap();
        bytes
    };
    let expected = export("native", runner.bin());
    let native: Value = serde_json::from_slice(&expected).unwrap();
    for (canonical, legacy, value) in [
        ("is_imex", "is_ixex", "1"),
        ("imex_nozzle_clearance_x", "ixex_carriage_width_x", "7"),
        ("imex_nozzle_clearance_y", "ixex_carriage_width_y", "11"),
    ] {
        assert_eq!(native[canonical], value, "Native {canonical} at {root:?}");
        assert!(native.get(legacy).is_none(), "Native exported {legacy}");
    }
    let actual = export("ares", std::path::Path::new(env!("CARGO_BIN_EXE_ares")));
    let ares: Value = serde_json::from_slice(&actual).unwrap();
    for key in [
        "is_imex",
        "imex_nozzle_clearance_x",
        "imex_nozzle_clearance_y",
    ] {
        assert_eq!(ares[key], native[key], "saved {key} at {root:?}");
    }
    assert!(
        actual == expected,
        "saved settings member differs at {root:?}"
    );
}

#[test]
fn invalid_imex_script_names_the_failed_custom_gcode_template() {
    let input = project(true, "{undefined_mode_value}", ("", "", ""));
    let Some(result) = slice_pair("imex-invalid-script", &input, 156) else {
        return;
    };
    for (output, gcode) in result {
        assert!(gcode.is_none());
        let diagnostic = format!(
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(
            diagnostic.contains("imex_mode_gcode Parsing error at line 1: Not a variable name")
        );
    }
}
