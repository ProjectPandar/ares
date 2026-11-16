//! Failure inventory: dormant-tool ranges lose moves, binary searches choose
//! another line, or pre-heating injection changes complete exported bytes.
use super::{generator_only, strict};
use crate::runner::OrcaRunner;
use ares_core::{ConfigOverrides, GenerationMetadata, SlicerContext};
use serde_json::Value;
use std::{
    fs,
    io::{Cursor, Read, Write},
};

const PROJECT: &[u8] = include_bytes!(
    "../../../../../../tests/parity/preheat-normal-move-durations/x2d-two-prisms.3mf"
);

fn project(tool_change: usize) -> Vec<u8> {
    let mut archive = zip::ZipArchive::new(Cursor::new(PROJECT)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut config: Value = serde_json::from_slice(&bytes).unwrap();
            config["enable_pre_heating"] = "1".into();
            config["filament_ids"] = serde_json::json!(["", ""]);
            config["thumbnails"] = "48x48/PNG,300x300/PNG".into();
            config["machine_start_gcode"] = "; runtime heating sequence\nG90\nM83\n".into();
            config["layer_change_gcode"] = "; runtime layer change\n".into();
            config["machine_end_gcode"] = "; runtime end\nM400\n".into();
            let original = config["change_filament_gcode"].as_str().unwrap();
            let mut script = format!("{{if toolchange_count == {tool_change}}}\nM83\n");
            for _ in 0..1_048_576 {
                script.push_str("G1 E0.001 F60\n");
            }
            script.push_str("M400\n{endif}\n");
            script.push_str(original);
            config["change_filament_gcode"] = script.into();
            bytes = serde_json::to_vec(&config).unwrap();
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

async fn compare(tool_change: usize) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let project = project(tool_change);
    let started = std::time::Instant::now();
    let mut archive = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    let config: Value =
        serde_json::from_reader(archive.by_name("Metadata/project_settings.config").unwrap())
            .unwrap();
    let mut edits = serde_json::Map::new();
    for key in [
        "enable_pre_heating",
        "machine_start_gcode",
        "layer_change_gcode",
        "machine_end_gcode",
        "change_filament_gcode",
        "thumbnails",
    ] {
        edits.insert(key.into(), config[key].clone());
    }

    let profiles = crate::runner::repo_root().join("OrcaSlicer/resources/profiles");
    let mut catalog = serde_json::json!({"vendors": [crate::runner::profile_entry::vendor(&profiles, "OrcaFilamentLibrary"), crate::runner::profile_entry::vendor(&profiles, "BBL")], "imports": []});
    for vendor in catalog["vendors"].as_array_mut().unwrap() {
        for fields in vendor["files"].as_object_mut().unwrap().values_mut() {
            if fields["type"] == "filament"
                && config["filament_settings_id"]
                    .as_array()
                    .unwrap()
                    .contains(&fields["name"])
            {
                fields["filament_id"] = "".into();
            }
            if fields["type"] == "machine" && fields["name"] == config["printer_settings_id"] {
                fields.as_object_mut().unwrap().extend(edits.clone());
            }
        }
    }
    let selection = serde_json::json!({"machine": config["printer_settings_id"], "process": config["print_settings_id"], "filaments": config["filament_settings_id"]});
    let presets = crate::runner::profile_entry::native_user_presets::resolved(
        &runner.work,
        &profiles,
        &catalog,
        &selection,
    );

    let input_path = runner.work.join("input.3mf");
    fs::write(&input_path, &project).unwrap();
    let directory = runner.work.join("native");
    fs::create_dir_all(&directory).unwrap();
    crate::runner::stages::run_live(
        runner.bin(),
        [
            "--datadir".to_owned(),
            presets.datadir.to_str().unwrap().to_owned(),
            "--load-settings".to_owned(),
            crate::runner::join_paths(presets.settings.iter()),
            "--load-filaments".into(),
            crate::runner::join_paths(presets.filaments.iter()),
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            directory.to_str().unwrap().to_owned(),
            input_path.to_str().unwrap().to_owned(),
        ],
        crate::runner::stages::Stage::Slice,
        &runner.work,
    )
    .unwrap();
    let native = fs::read(directory.join("plate_1.gcode")).unwrap();
    let catalog_json = serde_json::to_string(&catalog).unwrap();
    let defaults = ConfigOverrides::default();
    let actual = ares_core::slice_with_profiles_processed(
        project,
        ares_core::ProfileSliceRequest {
            filename: "input.3mf",
            catalog_json: &catalog_json,
            selection_json: r#"{"from_project":true,"uptodate":false}"#,
            plate: None,
            overrides: &defaults,
            placement: ares_core::Placement::None,
        },
        GenerationMetadata::deterministic(1980, 1, 1, 0, 0, 0),
        SlicerContext::new(0),
    )
    .await
    .unwrap();
    fs::write(runner.work.join("complete-native.gcode"), &native).unwrap();
    fs::write(runner.work.join("complete-ares.gcode"), &actual.gcode).unwrap();
    let expected = generator_only(&native, "OrcaSlicer");
    let observed = generator_only(&actual.gcode, "Ares");
    strict(
        &runner,
        "runtime-generated two-tool project",
        &expected,
        &observed,
    );
    fs::write(
        runner.work.join("elapsed.txt"),
        format!("{} seconds\n", started.elapsed().as_secs_f64()),
    )
    .unwrap();
}

#[tokio::test]
async fn inactive_tool_heating_preserves_a_long_second_tool_change() {
    compare(2).await;
}

#[tokio::test]
async fn inactive_tool_heating_preserves_a_long_third_tool_change() {
    compare(3).await;
}
