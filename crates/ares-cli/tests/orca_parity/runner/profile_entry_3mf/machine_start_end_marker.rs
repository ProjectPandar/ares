//! `GCode.cpp:4379-4386`: emit the machine-start boundary under the effective preheat flag.
use crate::{
    presets::VendorProfiles,
    runner::{OrcaRunner, application, cache_identity},
};
use serde_json::json;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

#[derive(Clone, Copy)]
enum Variant {
    Stock,
    ExplicitFalse,
    InheritedH2d,
    EmptyStart,
    Mk4Default,
}

#[test]
fn h2d_02_complete_output() {
    run("Bambu Lab H2D 0.2 nozzle", Variant::Stock);
}
#[test]
fn h2d_04_complete_output() {
    run("Bambu Lab H2D 0.4 nozzle", Variant::Stock);
}
#[test]
fn h2d_06_complete_output() {
    run("Bambu Lab H2D 0.6 nozzle", Variant::Stock);
}
#[test]
fn h2d_08_complete_output() {
    run("Bambu Lab H2D 0.8 nozzle", Variant::Stock);
}
#[test]
fn x2d_02_complete_output() {
    run("Bambu Lab X2D 0.2 nozzle", Variant::Stock);
}
#[test]
fn x2d_04_complete_output() {
    run("Bambu Lab X2D 0.4 nozzle", Variant::Stock);
}
#[test]
fn x2d_06_complete_output() {
    run("Bambu Lab X2D 0.6 nozzle", Variant::Stock);
}
#[test]
fn x2d_08_complete_output() {
    run("Bambu Lab X2D 0.8 nozzle", Variant::Stock);
}
#[test]
fn explicit_false_omits_the_marker_complete_output() {
    run("Bambu Lab H2D 0.4 nozzle", Variant::ExplicitFalse);
}
#[test]
fn absent_h2d_override_inherits_the_enabled_preset_complete_output() {
    run("Bambu Lab H2D 0.4 nozzle", Variant::InheritedH2d);
}
#[test]
fn enabled_empty_start_still_emits_the_marker_complete_output() {
    run("Bambu Lab H2D 0.4 nozzle", Variant::EmptyStart);
}
#[test]
fn absent_mk4_flag_uses_the_false_default_complete_output() {
    run("Prusa MK4 0.4 nozzle", Variant::Mk4Default);
}

fn run(printer: &str, variant: Variant) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let supply = application::project_supply()
        .unwrap()
        .expect("matched preset catalog");
    let original = if matches!(variant, Variant::Mk4Default) {
        include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../tests/parity/extruded-weight-totals/mk4-stock.3mf"
        ))
        .to_vec()
    } else {
        let profiles = VendorProfiles::load(
            &crate::runner::repo_root().join("OrcaSlicer/resources/profiles"),
            "BBL",
        )
        .unwrap();
        let selection = crate::select_printer(&profiles, "BBL", printer).unwrap();
        crate::export_selection_case(
            &runner,
            &profiles,
            &selection,
            &crate::runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap()
        .project
    };
    let mut settings = application::settings(&original).unwrap();
    match variant {
        Variant::Stock | Variant::Mk4Default => {}
        Variant::ExplicitFalse => {
            settings.insert("enable_pre_heating".to_owned(), json!("0"));
        }
        Variant::InheritedH2d => {
            settings.remove("enable_pre_heating");
        }
        Variant::EmptyStart => {
            settings.insert("machine_start_gcode".to_owned(), json!(""));
        }
    }
    let mut unchanged_entries = serde_json::Map::new();
    let input = if matches!(variant, Variant::Stock | Variant::Mk4Default) {
        original.clone()
    } else {
        let mut archive = zip::ZipArchive::new(Cursor::new(&original)).unwrap();
        let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index).unwrap();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            if entry.name() == "Metadata/project_settings.config" {
                bytes = serde_json::to_vec_pretty(&settings).unwrap();
            } else {
                unchanged_entries.insert(
                    entry.name().to_owned(),
                    json!(cache_identity::raw_digest(&bytes)),
                );
            }
            writer
                .start_file(
                    entry.name(),
                    zip::write::SimpleFileOptions::default()
                        .compression_method(entry.compression()),
                )
                .unwrap();
            writer.write_all(&bytes).unwrap();
        }
        writer.finish().unwrap().into_inner()
    };
    let exported = runner.fixture_case(printer, input).unwrap();
    let root = exported.project_path.parent().unwrap();
    let native = runner.slice_selected_plate_case(&exported, 1).unwrap();
    fs::write(root.join("native.gcode"), &native.reference).unwrap();
    fs::write(root.join("catalog.json"), &supply.catalog).unwrap();
    let selection = r#"{"from_project":true,"uptodate":false}"#;
    fs::write(root.join("selection.json"), selection).unwrap();
    let enabled = !matches!(variant, Variant::ExplicitFalse | Variant::Mk4Default);
    let text = std::str::from_utf8(&native.reference).unwrap();
    let footer = format!("; enable_pre_heating = {}", u8::from(enabled));
    assert!(
        text.lines().any(|line| line == footer),
        "Native effective flag premise: {root:?}"
    );
    let lines = text.lines().collect::<Vec<_>>();
    let markers = lines
        .iter()
        .enumerate()
        .filter_map(|(i, line)| (*line == "; MACHINE_START_GCODE_END").then_some(i))
        .collect::<Vec<_>>();
    assert_eq!(
        markers.len(),
        usize::from(enabled),
        "Native marker premise: {root:?}"
    );
    if enabled {
        assert!(
            lines[markers[0] + 1..].contains(&"; filament start gcode"),
            "Native marker order: {root:?}"
        );
    }
    if matches!(variant, Variant::EmptyStart) {
        assert!(
            text.lines().any(|line| line == "; machine_start_gcode = "),
            "Native empty start premise: {root:?}"
        );
    }
    fs::write(
        root.join("native-premise.json"),
        serde_json::to_vec_pretty(&json!({
            "printer": printer,
            "input_sha256": cache_identity::raw_digest(&exported.project),
            "original_sha256": cache_identity::raw_digest(&original),
            "positive_input_unchanged": matches!(variant, Variant::Stock | Variant::Mk4Default),
            "unchanged_entries_sha256": unchanged_entries,
            "raw_enable_pre_heating": settings.get("enable_pre_heating"),
            "effective_enable_pre_heating": enabled,
            "empty_start": matches!(variant, Variant::EmptyStart),
            "Native_marker_lines_zero_based": markers,
            "catalog_sha256": supply.digest,
            "selection": selection,
            "runner_work": runner.work,
        }))
        .unwrap(),
    )
    .unwrap();
    let overrides = ares_core::ConfigOverrides::default();
    let api = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::slice_with_profiles(
            &exported.project,
            ares_core::ProfileSliceRequest {
                filename: "input.3mf",
                catalog_json: &supply.catalog,
                selection_json: selection,
                plate: Some(1),
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ));
    if let Ok(bytes) = &api {
        fs::write(root.join("ares.api.gcode"), bytes).unwrap();
    }
    let cli_path = root.join("ares.cli.gcode");
    let cli = application::cli_command()
        .args(["slice", "--plate", "1", "-o"])
        .arg(&cli_path)
        .arg(&exported.project_path)
        .output()
        .unwrap();
    fs::write(root.join("cli.stdout"), &cli.stdout).unwrap();
    fs::write(root.join("cli.stderr"), &cli.stderr).unwrap();
    let api_difference = api.as_ref().ok().and_then(|bytes| {
        crate::golden::compare_ordered_bytes_generator_only(&native.reference, bytes).err()
    });
    let cli_bytes = cli.status.success().then(|| fs::read(&cli_path).unwrap());
    let cli_difference = cli_bytes.as_ref().and_then(|bytes| {
        crate::golden::compare_ordered_bytes_generator_only(&native.reference, bytes).err()
    });
    fs::write(root.join("result.json"), serde_json::to_vec_pretty(&json!({
        "printer": printer,
        "enabled": enabled,
        "public_entry": "ares_core::slice_with_profiles",
        "api_error": api.as_ref().err().map(ToString::to_string),
        "api_difference": api_difference,
        "cli_exit": cli.status.code(),
        "cli_difference": cli_difference,
        "native_premise": "PASS",
        "whole_output_pass": api.is_ok() && api_difference.is_none() && cli.status.success() && cli_difference.is_none(),
    })).unwrap()).unwrap();
    let api = api.unwrap_or_else(|error| panic!("API admission before bytes: {error:?}; {root:?}"));
    assert!(
        cli.status.success(),
        "CLI admission before bytes: {:?}; {root:?}",
        cli.stderr
    );
    crate::golden::compare_ordered_bytes_generator_only(&native.reference, &api)
        .unwrap_or_else(|difference| panic!("{difference}; {root:?}"));
    crate::golden::compare_ordered_bytes_generator_only(
        &native.reference,
        cli_bytes.as_ref().unwrap(),
    )
    .unwrap_or_else(|difference| panic!("{difference}; {root:?}"));
}
