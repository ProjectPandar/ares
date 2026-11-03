use super::*;
use std::{
    io::{Cursor, Read, Write},
    path::Path,
};

pub(super) fn verify(runner: &OrcaRunner, root: &Path, project: &[u8], keys: &[&str]) {
    let mut reader = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..reader.len() {
        let mut file = reader.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for key in keys {
                settings[*key] = json!("0");
            }
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        writer
            .start_file(file.name(), zip::write::SimpleFileOptions::default())
            .unwrap();
        writer.write_all(&bytes).unwrap();
    }
    let project = writer.finish().unwrap().into_inner();
    let directory = tempfile::Builder::new()
        .prefix("adaptive-volumetric-zero-rejection-")
        .tempdir_in(root)
        .unwrap()
        .keep();
    let input = directory.join("input.3mf");
    std::fs::write(&input, &project).unwrap();
    let error = runner::stages::run(
        runner.bin(),
        vec![
            "--slice".into(),
            "0".into(),
            "--outputdir".into(),
            directory.join("native-output").to_str().unwrap().into(),
            input.to_str().unwrap().into(),
        ],
        runner::stages::Stage::Slice,
        &directory,
    )
    .unwrap_err();
    let native = error.captured.as_ref().unwrap();
    let cli = crate::runner::application::cli_command()
        .args(["slice", "-o"])
        .arg(directory.join("ares.gcode"))
        .arg(&input)
        .output()
        .unwrap();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap();
    let api = runtime.block_on(crate::runner::application::slice_project_plate_validated(
        &project,
        ares_core::GenerationMetadata::default(),
        None,
        &ares_core::ConfigOverrides::default(),
        ares_core::SlicerContext::new(0),
    ));
    for (name, bytes) in [
        ("native.stderr", native.stderr.as_slice()),
        ("native.stdout", native.stdout.as_slice()),
        ("ares.stderr", cli.stderr.as_slice()),
        ("ares.stdout", cli.stdout.as_slice()),
    ] {
        std::fs::write(directory.join(name), bytes).unwrap();
    }
    std::fs::write(directory.join("result.json"), serde_json::to_vec_pretty(&json!({"native_code":native.code,"ares_code":cli.status.code(),"api":api.as_ref().map(Vec::len).map_err(ToString::to_string),"native_evidence":error.evidence})).unwrap()).unwrap();
    assert_eq!(cli.status.code(), native.code, "live CLI exit parity");
    assert_eq!(cli.stderr, native.stderr, "live CLI stderr parity");
    let error = api.unwrap_err();
    let ares_core::SliceError::InvalidValuesInProject(message) = error else {
        panic!("wrong validated API rejection: {error:?}");
    };
    assert_eq!(
        format!("Param values in 3mf/config error: \n{message}\n").as_bytes(),
        native.stderr
    );
    assert!(!directory.join("ares.gcode").exists());
}
