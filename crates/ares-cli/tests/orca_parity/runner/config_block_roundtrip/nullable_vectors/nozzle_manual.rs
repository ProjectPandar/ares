use super::input::settings_variant;
use crate::runner::{
    OrcaRunner, application,
    native_api_cli::NativeCase,
    repo_root,
    stages::{self, Stage},
};
use ares_core::{ConfigOverrides, GenerationMetadata, SliceError, SlicerContext};
use serde_json::{Value, json};
use std::{
    fs,
    io::{Cursor, Read, Write},
};

const ENDER: &str = "orca_cli_ender3.3mf";
const H2C: &str = "nozzle-id-diameters/scalars/h2c-inventory-contexts.3mf";
const H2C_HYBRID: &str = "filament-group-flush-mechanisms/h2c_manual_multi_nozzle_hybrid.3mf";

#[test]
fn single_head_nozzle_manual_keeps_unresolved_maps_and_exports_resolved_ids() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    for (name, settings, nozzle) in [
        (
            "single-head-plate-nozzle-manual",
            json!({"filament_map_mode": "Auto For Flush"}),
            1,
        ),
        (
            "single-head-missized-nozzle-manual",
            json!({
                "filament_map_mode": "Nozzle Manual",
                "filament_volume_map": ["0", "0"],
                "extruder_nozzle_stats": ["Standard#1"],
            }),
            1,
        ),
        (
            "single-head-resolved-nozzle-manual",
            json!({
                "filament_map_mode": "Nozzle Manual",
                "filament_volume_map": ["0"],
                "filament_nozzle_map": ["1"],
                "extruder_nozzle_stats": ["Standard#1"],
            }),
            0,
        ),
    ] {
        let input = variant(ENDER, settings, |xml| {
            replace_once(
                xml,
                "key=\"filament_map_mode\" value=\"Auto For Flush\"",
                "key=\"filament_map_mode\" value=\"Nozzle Manual\"",
            )
        });
        let case = NativeCase::slice_live(&runner, &input, name);
        assert!(
            case.native
                .contains("; filament_map_mode = Nozzle Manual\n")
        );
        assert!(
            case.native
                .contains(&format!("; filament_nozzle_map = {nozzle}\n"))
        );
        case.compare(metadata());
    }
}

#[test]
fn project_nozzle_manual_without_a_plate_mode_uses_the_native_default() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = variant(
        ENDER,
        json!({"filament_map_mode": "Nozzle Manual"}),
        |xml| {
            replace_once(
                xml,
                "<metadata key=\"filament_map_mode\" value=\"Auto For Flush\"/>",
                "",
            )
        },
    );
    let case = NativeCase::slice_live(&runner, &input, "project-nozzle-manual-plate-default-");
    assert!(
        case.native
            .contains("; filament_map_mode = Auto For Flush\n")
    );
    case.compare(metadata());
}

#[test]
fn nozzle_manual_assigns_distinct_nozzles_and_reuses_shared_labels() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    for (labels, exported) in [(["99", "42", "7"], "0,1,2"), (["99", "5", "5"], "0,1,1")] {
        let input = h2c_variant(&labels, ["Standard#1", "Standard#6"]);
        let case = NativeCase::slice_live(&runner, &input, "nozzle-manual-labels-");
        assert!(
            case.native
                .contains(&format!("; filament_nozzle_map = {exported}\n"))
        );
        case.compare(metadata());
    }
}

#[test]
fn plate_nozzle_manual_matches_inventory_to_the_plate_volume_choices() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let input = variant(
        H2C_HYBRID,
        json!({
            "filament_map_mode": "Auto For Flush",
            "filament_map": ["1", "1", "1"],
            "filament_volume_map": ["0", "0", "0"],
            "filament_nozzle_map": ["0", "2", "1"],
            "extruder_nozzle_stats": ["Standard#1|malformed|Unknown#9", "High Flow#2|Standard#2"],
        }),
        |xml| {
            let xml = replace_once(
                xml,
                "key=\"filament_map_mode\" value=\"Manual\"",
                "key=\"filament_map_mode\" value=\"Nozzle Manual\"",
            );
            replace_once(
                &xml,
                "<metadata key=\"filament_maps\" value=\"1 2 1\" />",
                "<metadata key=\"filament_maps\" value=\"1 2 2\" /><metadata key=\"filament_volume_maps\" value=\"3 1 2\" />",
            )
        },
    );
    let case = NativeCase::slice_live(&runner, &input, "plate-nozzle-manual-volume-");
    assert!(case.native.contains("; filament_volume_map = 0,1,0\n"));
    assert!(case.native.contains("; filament_nozzle_map = 0,3,1\n"));
    case.compare(metadata());
}

#[test]
fn nozzle_manual_without_a_complete_assignment_cannot_export_inventory_macros() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    for (labels, stats) in [
        (&["1"][..], ["Standard#1", "Standard#6"]),
        (&["0", "1", "2"][..], ["Standard#1", "Standard#1"]),
    ] {
        let input = h2c_variant(labels, stats);
        let root = tempfile::Builder::new()
            .prefix("nozzle-manual-empty-group-")
            .tempdir_in(crate::artifacts::root_from_env().unwrap())
            .unwrap()
            .keep();
        let path = root.join("input.3mf");
        fs::write(&path, &input).unwrap();
        let native = root.join("native");
        fs::create_dir(&native).unwrap();
        let error = stages::run_live(
            runner.bin(),
            [
                "--slice".to_owned(),
                "0".to_owned(),
                "--outputdir".to_owned(),
                native.to_str().unwrap().to_owned(),
                path.to_str().unwrap().to_owned(),
            ],
            Stage::Slice,
            &root,
        )
        .unwrap_err();
        let expected_code = if cfg!(unix) { 156 } else { -100 };
        assert_eq!(error.captured.unwrap().code, Some(expected_code));
        assert!(!native.join("plate_1.gcode").exists());

        let result = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap()
            .block_on(application::slice_project_plate_validated(
                &input,
                metadata(),
                None,
                &ConfigOverrides::default(),
                SlicerContext::new(1),
            ));
        assert!(
            matches!(result, Err(SliceError::CustomGCode { .. })),
            "{result:?}"
        );
        let output = root.join("ares.cli.gcode");
        let cli = application::cli_command()
            .args(["slice", "-o"])
            .arg(&output)
            .arg(&path)
            .output()
            .unwrap();
        fs::write(root.join("ares.cli.stderr"), &cli.stderr).unwrap();
        assert_eq!(cli.status.code(), Some(expected_code));
        assert!(!output.exists());
    }
}

fn h2c_variant(labels: &[&str], stats: [&str; 2]) -> Vec<u8> {
    variant(
        H2C,
        json!({
            "filament_map_mode": "Nozzle Manual",
            "filament_nozzle_map": labels,
            "extruder_nozzle_stats": stats,
        }),
        |xml| {
            replace_once(
                xml,
                "key=\"filament_map_mode\" value=\"Manual\"",
                "key=\"filament_map_mode\" value=\"Nozzle Manual\"",
            )
        },
    )
}

fn variant(name: &str, settings: Value, plate: impl Fn(&str) -> String) -> Vec<u8> {
    let source = fs::read(repo_root().join("tests/parity").join(name)).unwrap();
    let source = settings_variant(&source, settings.as_object().unwrap());
    let mut source = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).unwrap();
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(plate(&xml).as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}

fn replace_once(xml: &str, old: &str, new: &str) -> String {
    assert_eq!(xml.matches(old).count(), 1);
    xml.replace(old, new)
}

fn metadata() -> GenerationMetadata {
    GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0)
}
