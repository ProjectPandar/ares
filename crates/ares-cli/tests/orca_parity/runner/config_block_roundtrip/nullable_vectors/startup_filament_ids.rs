use std::{
    fs,
    io::{Cursor, Read, Write},
};

use crate::runner::{OrcaRunner, native_api_cli::NativeCase, repo_root};

#[test]
fn startup_filament_ids_match_native_for_initial_alias() {
    compare_x2d("initial_filament_id", 0);
}

#[test]
fn startup_filament_ids_match_native_for_current_alias() {
    compare_x2d("current_filament_id", 1);
}

#[test]
fn startup_filament_ids_match_native_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let base =
        fs::read(repo_root().join("tests/parity/shared-mesh-centering/nested-bfs.3mf")).unwrap();
    let project = with_script(
        &base,
        "; W4_STARTUP_IDS {initial_filament_id} {current_filament_id}\n",
    );
    let case = NativeCase::slice(&runner, &project, "startup-filament-ids-complete-");
    assert_eq!(marker(&case.native), "; W4_STARTUP_IDS 0 0");
    let (api, cli) = case.compare(ares_core::GenerationMetadata::deterministic(
        2026, 8, 27, 0, 0, 0,
    ));
    assert_eq!(api, cli, "raw API/CLI equality");
}

fn compare_x2d(alias: &str, expected: u32) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let base = fs::read(repo_root().join("tests/parity/filament-group-flush-mechanisms/x2d_pla_auto_map_first_filament_head1_only.3mf")).unwrap();
    let script = format!("; W4_STARTUP_IDS {{{alias}}} {{initial_extruder}}\n");
    let mut project = with_script(&base, &script);
    if expected != 0 {
        project = use_second_filament(&project);
    }
    let case = NativeCase::slice(&runner, &project, &format!("startup-{alias}-"));
    let native = marker(&case.native);
    assert_eq!(native, format!("; W4_STARTUP_IDS {expected} {expected}"));
    let output = case.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(&output)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(case.root.join("ares.cli.stderr"), &cli.stderr).unwrap();
    assert!(
        cli.status.success(),
        "CLI at {:?}: {}",
        case.root,
        String::from_utf8_lossy(&cli.stderr)
    );
    assert_eq!(marker(&fs::read_to_string(output).unwrap()), native);
}

fn marker(stream: &str) -> &str {
    let mut markers = stream
        .lines()
        .filter(|line| line.starts_with("; W4_STARTUP_IDS "));
    let line = markers.next().unwrap();
    assert!(markers.next().is_none());
    line
}

fn with_script(project: &[u8], script: &str) -> Vec<u8> {
    super::input::settings_variant(
        project,
        &[("machine_start_gcode".to_owned(), script.into())]
            .into_iter()
            .collect(),
    )
}

fn use_second_filament(project: &[u8]) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/model_settings.config" {
            let mut xml = String::new();
            entry.read_to_string(&mut xml).unwrap();
            let xml = xml
                .replace(
                    "key=\"extruder\" value=\"1\"",
                    "key=\"extruder\" value=\"2\"",
                )
                .replace(
                    "key=\"extruder\" value=\"3\"",
                    "key=\"extruder\" value=\"2\"",
                );
            writer
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            writer.write_all(xml.as_bytes()).unwrap();
        } else {
            writer.raw_copy_file(entry).unwrap();
        }
    }
    writer.finish().unwrap().into_inner()
}
