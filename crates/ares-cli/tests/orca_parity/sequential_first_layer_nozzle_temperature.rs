use crate::{OrcaRunner, runner::native_api_cli::NativeCase};
use ares_core::GenerationMetadata;
use serde_json::{Value, json};
use std::{fs, io::Cursor};
use zip::write::SimpleFileOptions;

const SEQUENTIAL: &[u8] =
    include_bytes!("../../../../tests/parity/byobject-transition-comments/comments-off.3mf");
const SECOND_TOOL: &[u8] = include_bytes!(
    "../../../../tests/parity/sequential-first-layer-nozzle-temperature/second-tool.3mf"
);
const RESTART: &str = "; Begin next object";
const TOOLLESS: &str = "G10 S220 ; set nozzle temperature";

#[test]
fn reprapfirmware_sequential_restart_sets_single_nozzle_without_tool_or_wait() {
    compare("0");
}

#[test]
fn reprapfirmware_sequential_restart_omits_tool_in_single_nozzle_multimaterial_mode() {
    compare("1");
}

#[test]
fn sequential_auto_for_flush_publishes_footer_maps_with_an_unused_tool() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let case = NativeCase::slice(&runner, SECOND_TOOL, "sequential-restart-nozzle-tool-");
    let expected = restart_nozzle_commands(&case.native);
    assert_eq!(expected.len(), 1);
    assert!(expected[0].starts_with("G10 S") && expected[0].contains(" P1 "));
    let (api, cli) = slice_plate(&case, SECOND_TOOL);
    assert_eq!(restart_nozzle_commands(&api), expected, "API restart");
    assert_eq!(restart_nozzle_commands(&cli), expected, "CLI restart");
    assert_footer_maps(&case.native, &api, &cli);
}

#[test]
fn by_layer_auto_for_flush_publishes_footer_maps_with_an_unused_tool() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = non_sequential_project();
    let case = NativeCase::slice(&runner, &project, "non-sequential-two-tool-footer-maps-");
    let (api, cli) = slice_plate(&case, &project);
    assert_footer_maps(&case.native, &api, &cli);
}

fn slice_plate(case: &NativeCase, project: &[u8]) -> (String, String) {
    let api = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project_plate_validated(
            project,
            GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            Some(1),
            &ares_core::ConfigOverrides::default(),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    fs::write(case.root.join("ares.api.gcode"), &api).unwrap();
    let cli_path = case.root.join("ares.cli.gcode");
    let cli = crate::runner::application::cli_command()
        .args(["slice", "--plate", "1", "-o"])
        .arg(&cli_path)
        .arg(case.root.join("input.3mf"))
        .output()
        .unwrap();
    fs::write(case.root.join("ares.cli.stdout"), cli.stdout).unwrap();
    fs::write(case.root.join("ares.cli.stderr"), cli.stderr).unwrap();
    assert!(cli.status.success(), "CLI at {:?}", case.root);
    (
        String::from_utf8(api).unwrap(),
        fs::read_to_string(cli_path).unwrap(),
    )
}

fn assert_footer_maps(native: &str, api: &str, cli: &str) {
    for (source, gcode) in [("Native", native), ("API", api), ("CLI", cli)] {
        let (_, footer) = gcode.split_once("; CONFIG_BLOCK_START\n").unwrap();
        let (footer, _) = footer.split_once("; CONFIG_BLOCK_END").unwrap();
        for (field, value) in [
            ("filament_map_mode", "Auto For Flush"),
            ("filament_map", "1,2"),
            ("filament_map_2", "0,1"),
            ("filament_nozzle_map", "0,1"),
            ("filament_volume_map", "0,0"),
        ] {
            let prefix = format!("; {field} = ");
            let values: Vec<_> = footer
                .lines()
                .filter_map(|line| line.strip_prefix(&prefix))
                .collect();
            assert_eq!(values, [value], "{source} {field}");
        }
    }
}

fn compare(semm: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = project(semm);
    let case = NativeCase::slice(&runner, &project, "sequential-restart-nozzle-");
    assert_eq!(restart_nozzle_commands(&case.native), [TOOLLESS]);
    let (api, _) = case.compare(GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0));
    assert_eq!(
        restart_nozzle_commands(std::str::from_utf8(&api).unwrap()),
        [TOOLLESS]
    );
}

fn project(semm: &str) -> Vec<u8> {
    rewrite_project(SEQUENTIAL, |config| {
        config["gcode_flavor"] = json!("reprapfirmware");
        config["machine_start_gcode"] = json!("G90\nM83");
        config["printing_by_object_gcode"] = json!(RESTART);
        config["single_extruder_multi_material"] = json!(semm);
        config["disable_m73"] = json!("1");
        config["nozzle_temperature_initial_layer"] = json!(["220"]);
        config["nozzle_temperature"] = json!(["210"]);
        config["printer_agent"] = json!("");
    })
}

fn non_sequential_project() -> Vec<u8> {
    rewrite_project(SECOND_TOOL, |config| {
        config["print_sequence"] = json!("by layer");
        config["printing_by_object_gcode"] = json!("");
    })
}

fn rewrite_project(source: &[u8], mut update: impl FnMut(&mut Value)) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(source)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() != "Metadata/project_settings.config" {
            output.raw_copy_file(entry).unwrap();
            continue;
        }
        let mut config: Value = serde_json::from_reader(entry).unwrap();
        update(&mut config);
        output
            .start_file(
                "Metadata/project_settings.config",
                SimpleFileOptions::DEFAULT,
            )
            .unwrap();
        serde_json::to_writer(&mut output, &config).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn restart_nozzle_commands(gcode: &str) -> Vec<&str> {
    let marker = format!("\n{RESTART}\n");
    let (previous, next) = gcode.split_once(&marker).unwrap();
    assert!(
        !next.contains(&marker),
        "expected exactly one object restart"
    );
    assert!(next.contains(";LAYER_CHANGE\n"));
    let (_, transition) = previous.rsplit_once(";LAYER_CHANGE\n").unwrap();
    transition
        .lines()
        .filter(|line| {
            ["G10 S", "M104 ", "M109 ", "M116"]
                .iter()
                .any(|prefix| line.starts_with(prefix))
        })
        .collect()
}
