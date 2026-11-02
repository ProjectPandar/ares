use super::super::{
    OrcaRunner, cache_identity,
    command::{self, Limits},
    profile_entry, repo_root,
    stages::Stage,
};
use serde_json::{Value, json};
use std::{
    io::{Cursor, Write},
    time::Duration,
};

const FIXTURE: &str = "tests/parity/spiral-vase-admission/two-object-by-object-apart.3mf";
const KEYS: [&str; 3] = [
    "bottom_color_penetration_layers",
    "top_color_penetration_layers",
    "extruder_clearance_dist_to_rod",
];

#[test]
fn project_preset_refresh_fills_missing_named_preset_values() {
    check("missing", false, false, false, ["3", "5", "56.5"]);
}

#[test]
fn project_preset_refresh_keeps_present_values_without_refresh() {
    check("retain", true, false, false, ["7", "8", "41"]);
}

#[test]
fn project_preset_refresh_protects_dirty_process_and_machine_slots() {
    check("dirty", true, true, false, ["7", "5", "41"]);
}

#[test]
fn project_preset_refresh_uptodate_refreshes_unlisted_values() {
    check("uptodate", true, false, true, ["3", "5", "56.5"]);
}

fn check(label: &str, authored: bool, dirty: bool, uptodate: bool, expected: [&str; 3]) {
    let Some(runner) = OrcaRunner::from_env().unwrap_or_else(|error| panic!("{error}")) else {
        return;
    };
    let root = repo_root();
    let input = std::fs::read(root.join(FIXTURE)).unwrap();
    let project = if authored {
        let mut settings = super::super::application::settings(&input).unwrap();
        for (key, value) in KEYS.into_iter().zip(["7", "8", "41"]) {
            settings.insert(key.to_owned(), json!(value));
        }
        if dirty {
            settings.insert(
                "different_settings_to_system".to_owned(),
                json!([KEYS[0], "", KEYS[2]]),
            );
        }
        replace_settings(&input, &settings)
    } else {
        input
    };
    let source = super::super::application::settings(&project).unwrap();
    for key in KEYS {
        assert_eq!(source.contains_key(key), authored);
    }
    let project_path = runner.work.join(format!("{label}.3mf"));
    std::fs::write(&project_path, &project).unwrap();
    let catalog = json!({
        "vendors": [
            profile_entry::vendor(&root.join("OrcaSlicer/resources/profiles"), "OrcaFilamentLibrary"),
            profile_entry::vendor(&root.join("OrcaSlicer/resources/profiles"), "BBL"),
        ],
        "imports": [],
    });
    let selection = json!({"from_project": true, "uptodate": uptodate});
    let datadir = profile_entry::native_user_presets::system(
        &runner.work,
        &root.join("OrcaSlicer/resources/profiles"),
        &["OrcaFilamentLibrary", "BBL"],
    );
    profile_entry::write_json(&runner.work.join("catalog.json"), &catalog);
    profile_entry::write_json(&runner.work.join("selection.json"), &selection);
    let producer = cache_identity::identify(runner.bin()).unwrap();
    profile_entry::write_json(
        &runner.work.join("native-producer.json"),
        &serde_json::to_value(producer.receipt).unwrap(),
    );
    std::fs::write(
        runner.work.join("input-sha256.txt"),
        cache_identity::raw_digest(&project),
    )
    .unwrap();

    let native = (0..2)
        .map(|repeat| {
            let output = runner.work.join(format!("native-settings-{repeat}.json"));
            let mut args = vec![
                "--datadir".to_owned(),
                datadir.to_string_lossy().into_owned(),
                "--export-settings".to_owned(),
                output.to_string_lossy().into_owned(),
            ];
            if uptodate {
                args.push("--uptodate".to_owned());
            }
            args.push(project_path.to_string_lossy().into_owned());
            command::run(
                runner.bin(),
                args,
                &[("LC_ALL", "C")],
                Limits {
                    stage: Stage::Export,
                    work: &runner.work,
                    timeout: Duration::from_secs(120),
                },
            )
            .unwrap_or_else(|error| panic!("Native export: {error}; {:?}", runner.work));
            serde_json::from_slice::<Value>(&std::fs::read(output).unwrap()).unwrap()
        })
        .collect::<Vec<_>>();
    assert_eq!(native[0], native[1], "Native repeat; {:?}", runner.work);
    assert_eq!(native[0]["version"], "2.5.0-beta");
    for (key, token) in KEYS.into_iter().zip(expected) {
        assert_eq!(
            native[0][key], token,
            "Native premise {key}; {:?}",
            runner.work
        );
    }

    let overrides = ares_core::ConfigOverrides::default();
    let catalog_json = catalog.to_string();
    let selection_json = selection.to_string();
    let result = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(ares_core::profile_settings(
            &project,
            ares_core::ProfileSliceRequest {
                filename: "project.3mf",
                catalog_json: &catalog_json,
                selection_json: &selection_json,
                plate: Some(1),
                overrides: &overrides,
                placement: ares_core::Placement::None,
            },
        ));
    let record = match &result {
        Ok(settings) => json!({"settings": serde_json::to_value(settings).unwrap()}),
        Err(error) => json!({"error": error.to_string()}),
    };
    profile_entry::write_json(&runner.work.join("public-result.json"), &record);
    profile_entry::write_json(
        &runner.work.join("comparison.json"),
        &json!({
            "label": label,
            "keys": KEYS,
            "native": KEYS.map(|key| native[0][key].clone()),
            "request": selection,
            "api": record,
        }),
    );
    let settings = result.unwrap_or_else(|error| {
        panic!("public project-derived request: {error}; {:?}", runner.work)
    });
    let settings = serde_json::to_value(settings).unwrap();
    for key in KEYS.into_iter().chain([
        "compatible_process_expression_group",
        "pellet_flow_coefficient",
        "filament_retract_after_wipe",
        "filament_retract_length_toolchange",
        "filament_retract_lift_above",
        "filament_retract_lift_below",
        "filament_retract_lift_enforce",
        "filament_retract_restart_extra_toolchange",
    ]) {
        assert_eq!(settings[key], native[0][key], "{key}; {:?}", runner.work);
    }
}

fn replace_settings(input: &[u8], settings: &serde_json::Map<String, Value>) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(input)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let entry = archive.by_index(index).unwrap();
        if entry.name() == "Metadata/project_settings.config" {
            output
                .start_file(entry.name(), zip::write::SimpleFileOptions::default())
                .unwrap();
            output
                .write_all(&serde_json::to_vec(settings).unwrap())
                .unwrap();
        } else {
            output.raw_copy_file(entry).unwrap();
        }
    }
    output.finish().unwrap().into_inner()
}
