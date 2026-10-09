//! `GCode.cpp:3481-3491` and `PlaceholderParser.cpp:827/915`.
use std::io::{Cursor, Read, Write};

use super::{ExportedCase, OrcaRunner};

#[test]
fn orca_parity_tiq2_footer_slot_index_differs_from_active_writer() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        crate::native_environment::skipped("TiQ2 footer: no optional OrcaRunner");
        return;
    };
    let input = include_bytes!("../../../../../tests/parity/validation/tiq2-type2-enabled.3mf");
    let mut archive = zip::ZipArchive::new(Cursor::new(input.as_slice())).unwrap();
    let mut rewritten = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..archive.len() {
        let mut file = archive.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
            settings["single_extruder_multi_material"] = "0".into();
            settings["nozzle_temperature"] = serde_json::json!(["211", "233"]);
            settings["filament_end_gcode"] = serde_json::json!([
                "; FOOTER_SLOT0 current={current_extruder} slot={filament_extruder_id} temp=[nozzle_temperature]\n",
                "; FOOTER_SLOT1 current={current_extruder} slot={filament_extruder_id} temp=[nozzle_temperature]\n"
            ]);
            settings["machine_end_gcode"] = "; FOOTER_MACHINE current={current_extruder}\n".into();
            bytes = serde_json::to_vec(&settings).unwrap();
        }
        rewritten
            .start_file(
                file.name(),
                zip::write::SimpleFileOptions::default().compression_method(file.compression()),
            )
            .unwrap();
        rewritten.write_all(&bytes).unwrap();
    }
    let project = rewritten.finish().unwrap().into_inner();
    let directory = tempfile::Builder::new()
        .prefix("tiq2-footer-slot-")
        .tempdir_in(crate::artifacts::root_from_env().unwrap())
        .unwrap()
        .keep();
    let project_path = directory.join("input.3mf");
    std::fs::write(&project_path, &project).unwrap();
    let output_dir = directory.join("native");
    std::fs::create_dir(&output_dir).unwrap();
    let case = runner
        .slice_case(&ExportedCase {
            label: "TiQ2/footer-slot-index".to_owned(),
            project: project.clone(),
            project_path,
            output_dir,
        })
        .unwrap();
    let actual = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(ares_core::slice_project(
            &project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    std::fs::write(directory.join("orca.gcode"), &case.reference).unwrap();
    std::fs::write(directory.join("ares.gcode"), &actual).unwrap();
    let reference = footer_records(&case.reference);
    assert_eq!(
        reference,
        [
            "; FOOTER_SLOT0 current=0 slot=0 temp=211",
            "; FOOTER_SLOT1 current=0 slot=1 temp=233",
            "; FOOTER_MACHINE current=0",
        ]
    );
    let actual = footer_records(&actual);
    std::fs::write(
        directory.join("footer-context.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "reference": reference,
            "actual": actual,
        }))
        .unwrap(),
    )
    .unwrap();
    assert_eq!(actual, reference, "runtime evidence: {}", directory.display());
}

fn footer_records(bytes: &[u8]) -> Vec<&str> {
    let text = std::str::from_utf8(bytes).unwrap();
    let text = text.split("; CONFIG_BLOCK_START").next().unwrap();
    let start = text.rfind("; FOOTER_SLOT0").unwrap();
    text[start..]
        .lines()
        .filter(|line| line.starts_with("; FOOTER_"))
        .collect()
}
