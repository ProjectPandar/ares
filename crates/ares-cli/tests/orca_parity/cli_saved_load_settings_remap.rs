#![cfg(target_os = "linux")]

use std::{
    ffi::OsString,
    fs,
    io::{self, Cursor},
    path::Path,
};

use serde_json::{Value, json};
use zip::{ZipArchive, ZipWriter, write::SimpleFileOptions};

use super::{
    cli_no_check::{gcode, inventory, run},
    direct_stl_slice::block_stl,
};
use crate::{
    artifacts, load_settings_cli::preset_files, native_wall_clock::result_comparison_bytes,
    runner::OrcaRunner,
};

const MACHINE: &str = "M117 Replacement machine";
const FILAMENT: &str = "M117 Replacement filament";

#[test]
fn cli_saved_load_settings_remaps_one_filament_and_disables_prime_tower() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let root = tempfile::Builder::new()
        .prefix("cli-saved-load-settings-remap-")
        .tempdir_in(artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let presets = preset_files(&root, "Creality", "Creality Ender-3 0.4 nozzle");
    let settings = presets[..2].join(";");
    let filament = &presets[2];
    let load = ["--load-settings", &settings, "--load-filaments", filament];
    let stl = root.join("block.stl");
    fs::write(&stl, block_stl(108.0)).unwrap();
    let export = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--export-3mf",
        "saved.3mf",
    ]
    .into_iter()
    .chain(load)
    .chain([stl.to_str().unwrap()])
    .map(OsString::from)
    .collect::<Vec<_>>();
    assert!(
        run(runner.bin(), &root, "export", &export).status.success(),
        "{root:?}"
    );
    let project = root.join("saved.3mf");
    repack_project(&root.join("export/cwd/out/saved.3mf"), &project);
    let machine = replacement_preset(&presets[0], |config| {
        config["machine_start_gcode"] = json!(format!(
            "{}\n{MACHINE}",
            config["machine_start_gcode"].as_str().unwrap()
        ));
    });
    let process = replacement_preset(&presets[1], |config| {
        config["wall_loops"] = json!("3");
        config["timelapse_type"] = json!("1");
        config["enable_prime_tower"] = json!("1");
    });
    let filament = replacement_preset(filament, |config| {
        config["filament_start_gcode"] = json!([FILAMENT]);
        // Use JSON arrays for string vectors in the synthetic preset.
        for key in [
            "default_filament_colour",
            "filament_notes",
            "filament_ramming_parameters",
        ] {
            let token: String = serde_json::from_str(config[key].as_str().unwrap()).unwrap();
            config[key] = json!([token]);
        }
    });
    let settings = format!("{machine};{process}");
    let load = ["--load-settings", &settings, "--load-filaments", &filament];
    let slice = [
        "--debug",
        "0",
        "--outputdir",
        "out",
        "--slice",
        "1",
        "--arrange",
        "0",
    ];
    let input = project.to_str().unwrap();
    let retained = slice
        .into_iter()
        .chain([input])
        .map(OsString::from)
        .collect::<Vec<_>>();
    assert!(
        run(runner.bin(), &root, "retained", &retained)
            .status
            .success(),
        "{root:?}"
    );
    let retained_gcode = gcode(&root.join("retained/cwd"));
    let retained_text = std::str::from_utf8(&retained_gcode).unwrap();
    assert!(retained_text.contains("; enable_prime_tower = 1\n"));
    assert!(!retained_text.contains(MACHINE));
    assert!(!retained_text.contains(FILAMENT));

    let argv = slice
        .into_iter()
        .chain(load)
        .chain([input])
        .map(OsString::from)
        .collect::<Vec<_>>();
    fs::write(root.join("argv.json"), serde_json::to_vec(&argv).unwrap()).unwrap();
    let native = run(runner.bin(), &root, "native", &argv);
    let native_cwd = root.join("native/cwd");
    assert_eq!(native.status.code(), Some(0), "{root:?}");
    let expected = gcode(&native_cwd);
    let native_text = std::str::from_utf8(&expected).unwrap();
    assert!(native_text.lines().any(|line| line == MACHINE), "{root:?}");
    assert!(native_text.lines().any(|line| line == FILAMENT), "{root:?}");
    assert!(native_text.contains("; timelapse_type = 1\n"), "{root:?}");
    assert!(
        native_text.contains("; enable_prime_tower = 0\n"),
        "{root:?}"
    );
    assert!(native_text.contains("; wall_loops = 3\n"), "{root:?}");
    assert_eq!(
        inventory(&native_cwd),
        ["out/", "out/plate_1.gcode", "out/result.json"]
    );

    let ares = run(Path::new(env!("CARGO_BIN_EXE_ares")), &root, "ares", &argv);
    let actual_cwd = root.join("ares/cwd");
    assert_eq!(ares.status.code(), native.status.code(), "{root:?}");
    assert_eq!(ares.stderr, native.stderr, "{root:?}");
    assert_eq!(ares.stdout, native.stdout, "{root:?}");
    assert_eq!(inventory(&actual_cwd), inventory(&native_cwd), "{root:?}");
    let report =
        |cwd: &Path| result_comparison_bytes(&fs::read(cwd.join("out/result.json")).unwrap());
    assert!(
        report(&actual_cwd) == report(&native_cwd),
        "complete report differs: {root:?}"
    );
    assert!(
        gcode(&actual_cwd) == expected,
        "complete G-code differs: {root:?}"
    );
}

fn replacement_preset(path: &str, edit: impl FnOnce(&mut Value)) -> String {
    let mut config: Value = serde_json::from_slice(&fs::read(path).unwrap()).unwrap();
    edit(&mut config);
    let path = Path::new(path);
    let replacement = path.with_file_name(format!(
        "replacement-{}",
        path.file_name().unwrap().to_string_lossy()
    ));
    fs::write(&replacement, serde_json::to_vec(&config).unwrap()).unwrap();
    replacement.into_os_string().into_string().unwrap()
}

fn repack_project(exported: &Path, project: &Path) {
    let mut source = ZipArchive::new(Cursor::new(fs::read(exported).unwrap())).unwrap();
    let mut archive = ZipWriter::new(Cursor::new(Vec::new()));
    let options = SimpleFileOptions::default();
    for index in 0..source.len() {
        let mut entry = source.by_index(index).unwrap();
        if entry.is_dir() {
            archive.add_directory(entry.name(), options).unwrap();
            continue;
        }
        let settings = entry.name() == "Metadata/project_settings.config";
        archive.start_file(entry.name(), options).unwrap();
        if settings {
            let mut config: Value = serde_json::from_reader(entry).unwrap();
            config["timelapse_type"] = json!("1");
            config["enable_prime_tower"] = json!("1");
            config["compatible_process_expression_group"] = json!([""]);
            config["pellet_flow_coefficient"] = json!(["0.4157"]);
            serde_json::to_writer(&mut archive, &config).unwrap();
        } else {
            io::copy(&mut entry, &mut archive).unwrap();
        }
    }
    fs::write(project, archive.finish().unwrap().into_inner()).unwrap();
}
