#[path = "stock_project_whole_file/golden.rs"]
mod golden;
#[path = "../../../tests/support/native_environment.rs"]
mod native_environment;

use ares_core::{
    ConfigOverrides, GenerationMetadata, SlicerContext, load_project, project_config_substitutions,
    slice_project_plate_validated,
};
use std::{
    fs,
    io::{Cursor, Read, Write},
    process::Command,
};

const SOURCE: &[u8] = include_bytes!("../../../tests/parity/orca_cli_ender3.3mf");
const SEQUENCES: &str = "Metadata/filament_sequence.json";

fn archive_variant(removed: Option<&str>, sequences: Option<&str>) -> Vec<u8> {
    let mut source = zip::ZipArchive::new(Cursor::new(SOURCE)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        if Some(entry.name()) == removed {
            continue;
        }
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        if entry.name() == SEQUENCES
            && let Some(text) = sequences
        {
            bytes = text.as_bytes().to_vec();
        }
        writer
            .start_file(entry.name(), zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn filament_sequence_without_nozzle_sequence_loads() {
    load_project(archive_variant(
        None,
        Some(r#"{"plate_1":{"sequence":[1]}}"#),
    ))
    .unwrap();
}

#[test]
fn filament_sequence_accepts_dynamic_key_and_ignores_unrelated_members() {
    for text in [
        r#"{"plate_1":{"filament_sequence":[0,1],"nozzle_sequence":[0],"optimal_assignment":[1,-1]}}"#,
        r#"{"plate_1":{"sequence":[],"nozzle_sequence":[]},"plate_2":{"sequence":["invalid"],"nozzle_sequence":[]},"other":false}"#,
        r#"{"plate_1":false}"#,
        r#"{"plate_1":{"sequence":["ignored"],"nozzle_sequence":null}}"#,
        r#"{"plate_1":{"filament_sequence":null,"sequence":["ignored"],"nozzle_sequence":[]}}"#,
        r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":false}}"#,
        "null",
    ] {
        load_project(archive_variant(None, Some(text))).unwrap();
    }
}

#[test]
fn malformed_sequence_json_and_invalid_array_items_fail_import() {
    for text in [
        "{",
        "",
        r#"{"plate_1":{"sequence":["invalid"],"nozzle_sequence":[]}}"#,
        r#"{"plate_1":{"sequence":[],"nozzle_sequence":[false]}}"#,
        r#"{"plate_1":{"sequence":[],"nozzle_sequence":[],"optimal_assignment":[null]}}"#,
    ] {
        let error = load_project(archive_variant(None, Some(text))).unwrap_err();
        assert!(error.to_string().contains("filament sequences"), "{error}");
    }
}

#[test]
fn optional_project_metadata_entries_may_be_absent() {
    for path in [
        SEQUENCES,
        "Metadata/slice_info.config",
        "Metadata/model_settings.config",
        "Metadata/project_settings.config",
    ] {
        let input = archive_variant(Some(path), None);
        let project = load_project(&input).unwrap();
        assert!(
            !project.objects().is_empty(),
            "geometry survives absent {path}"
        );
        if path == "Metadata/model_settings.config" {
            assert!(project.plates().is_empty());
        }
        if path == "Metadata/project_settings.config" {
            assert!(project_config_substitutions(&input).unwrap().is_empty());
        }
    }
}

#[tokio::test]
async fn optional_project_metadata_slices_like_native() {
    let Some(bin) = native_environment::binary("ARES_ORCA_BIN") else {
        return;
    };
    let root = std::env::var_os("ARES_PARITY_ARTIFACT_DIR")
        .map(std::path::PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    fs::create_dir_all(&root).unwrap();
    let directory = tempfile::Builder::new()
        .prefix("filament-sequence-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    for (name, removed, sequence) in [
        ("incomplete", None, Some(r#"{"plate_1":{"sequence":[1]}}"#)),
        (
            "dynamic",
            None,
            Some(r#"{"plate_1":{"filament_sequence":[1],"nozzle_sequence":[0]}}"#),
        ),
        (
            "unknown",
            None,
            Some(r#"{"plate_1":{"sequence":[],"nozzle_sequence":[]},"plate_2":false,"other":17}"#),
        ),
        ("nonobject", None, Some(r#"{"plate_1":false}"#)),
        ("absent-sequences", Some(SEQUENCES), None),
        (
            "absent-model-settings",
            Some("Metadata/model_settings.config"),
            None,
        ),
        (
            "absent-project-settings",
            Some("Metadata/project_settings.config"),
            None,
        ),
        (
            "absent-slice-info",
            Some("Metadata/slice_info.config"),
            None,
        ),
    ] {
        let input = archive_variant(removed, sequence);
        let folder = directory.join(name);
        fs::create_dir(&folder).unwrap();
        let path = folder.join("input.3mf");
        fs::write(&path, &input).unwrap();
        let output = Command::new(&bin)
            .args(["--slice", "1", "--outputdir"])
            .arg(&folder)
            .arg(&path)
            .output()
            .unwrap();
        fs::write(folder.join("native.stdout"), &output.stdout).unwrap();
        fs::write(folder.join("native.stderr"), &output.stderr).unwrap();
        if name == "absent-project-settings" {
            assert!(!output.status.success());
            let error = slice_project_plate_validated(
                &input,
                GenerationMetadata::default(),
                Some(1),
                &ConfigOverrides::default(),
                SlicerContext::new(0),
            )
            .await
            .unwrap_err();
            assert_eq!(
                error.to_string(),
                String::from_utf8(output.stderr).unwrap().trim()
            );
            fs::write(folder.join("ares-error.txt"), error.to_string()).unwrap();
            continue;
        }
        assert!(
            output.status.success(),
            "{name}: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let actual = slice_project_plate_validated(
            &input,
            GenerationMetadata::default(),
            Some(1),
            &ConfigOverrides::default(),
            SlicerContext::new(0),
        )
        .await
        .unwrap();
        fs::write(folder.join("ares.gcode"), &actual).unwrap();
        golden::compare_ordered_bytes_generator_only(
            &fs::read(folder.join("plate_1.gcode")).unwrap(),
            &actual,
        )
        .unwrap();
    }
}
