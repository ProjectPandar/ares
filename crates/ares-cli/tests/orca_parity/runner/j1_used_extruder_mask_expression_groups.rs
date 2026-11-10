use super::OrcaRunner;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[test]
fn j1_compatible_expression_groups_match_native() {
    run_compatible_expression_group_case(
        None,
        2,
        false,
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"
        ),
    );
}

#[test]
fn j1_compatible_expression_groups_preserve_nonempty_filament_conditions() {
    run_compatible_expression_group_case(
        Some("J1_NONEMPTY_CONDITION".to_owned()),
        2,
        false,
        include_bytes!(
            "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"
        ),
    );
}

#[test]
fn j1_compatible_expression_groups_extend_existing_short_vectors() {
    let project = with_short_expression_groups(include_bytes!(
        "../../../../../tests/parity/j1-used-extruder-follows-emitted-tools/both-used.3mf"
    ));
    run_compatible_expression_group_case(
        Some("J1_SHORT_VECTOR_CONDITION".to_owned()),
        3,
        true,
        &project,
    );
}

fn run_compatible_expression_group_case(
    condition: Option<String>,
    filament_count: usize,
    expect_native_rejection: bool,
    project: &[u8],
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let directory = tempfile::Builder::new()
        .prefix("j1-compatible-expression-groups-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let input_path = directory.join("input.3mf");
    let output_dir = directory.join("native");
    fs::write(&input_path, project).unwrap();
    fs::create_dir(&output_dir).unwrap();
    let filament_source = include_bytes!(
        "../../../../../tests/parity/validation/load-settings-3mf/compatible-filament-0.json"
    );
    let mut filament = serde_json::from_slice::<serde_json::Value>(filament_source).unwrap();
    if let Some(condition) = condition.as_deref() {
        filament["compatible_printers_condition"] = condition.into();
        filament["compatible_prints_condition"] = condition.into();
    }
    let filament_paths = (0..filament_count)
        .map(|index| {
            let path = directory.join(format!("filament-{index}.json"));
            let mut filament = filament.clone();
            filament["name"] = format!("J1 Filament {index}").into();
            filament["filament_id"] = format!("j1-filament-{index}").into();
            fs::write(&path, serde_json::to_vec(&filament).unwrap()).unwrap();
            path
        })
        .collect::<Vec<_>>();
    let filament_arg = filament_paths
        .iter()
        .map(|path| path.to_string_lossy())
        .collect::<Vec<_>>()
        .join(";");
    let native_result = crate::runner::stages::run(
        runner.bin(),
        [
            "--slice".to_owned(),
            "0".to_owned(),
            "--load-filaments".to_owned(),
            filament_arg.clone(),
            "--outputdir".to_owned(),
            output_dir.to_string_lossy().into_owned(),
            input_path.to_string_lossy().into_owned(),
        ],
        crate::runner::stages::Stage::Slice,
        &directory,
    );
    if expect_native_rejection {
        assert!(native_result.is_err());
    } else {
        native_result.unwrap();
    }
    let native =
        (!expect_native_rejection).then(|| fs::read(output_dir.join("plate_1.gcode")).unwrap());
    let ares_path = directory.join("ares.gcode");
    let cli = std::process::Command::new(env!("CARGO_BIN_EXE_ares"))
        .args(["slice", "--load-filaments"])
        .arg(&filament_arg)
        .args(["-o"])
        .arg(&ares_path)
        .arg(&input_path)
        .env_remove("ARES_PARITY_PROJECT_PRESET_CATALOG")
        .output()
        .unwrap();
    assert_eq!(cli.status.success(), !expect_native_rejection);
    if expect_native_rejection {
        return;
    }
    let native = native.unwrap();
    let actual = fs::read(&ares_path).unwrap();
    let expected = compatible_expression_group_lines(&native);
    let actual_lines = compatible_expression_group_lines(&actual);
    fs::write(directory.join("native.gcode"), &native).unwrap();
    assert_eq!(actual_lines, expected, "see {directory:?}");
}

fn with_short_expression_groups(source: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["compatible_machine_expression_group"] =
                serde_json::json!(["RAW_MACHINE_0", "RAW_MACHINE_1"]);
            settings["compatible_process_expression_group"] = serde_json::json!(["RAW_PROCESS"]);
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        output
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::DEFAULT.compression_method(file.compression()),
            )
            .unwrap();
        output.write_all(&bytes).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn compatible_expression_group_lines(bytes: &[u8]) -> Vec<String> {
    std::str::from_utf8(bytes)
        .unwrap()
        .lines()
        .filter(|line| {
            line.starts_with("; compatible_machine_expression_group = ")
                || line.starts_with("; compatible_process_expression_group = ")
        })
        .map(str::to_owned)
        .collect()
}
