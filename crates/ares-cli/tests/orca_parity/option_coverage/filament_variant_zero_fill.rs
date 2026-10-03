use super::*;
use crate::runner::CaseInputs;
use std::io::{Cursor, Read, Write};

fn shorten_payloads(project: &[u8]) -> Vec<u8> {
    let mut input = zip::ZipArchive::new(Cursor::new(project)).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..input.len() {
        let mut file = input.by_index(index).unwrap();
        let mut bytes = Vec::new();
        file.read_to_end(&mut bytes).unwrap();
        if file.name() == "Metadata/project_settings.config" {
            let mut settings: Value = serde_json::from_slice(&bytes).unwrap();
            for key in [
                "filament_z_hop_types",
                "filament_wipe",
                "volumetric_speed_coefficients",
            ] {
                settings[key].as_array_mut().unwrap().truncate(2);
            }
            bytes = serde_json::to_vec(&settings).unwrap();
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

#[test]
fn orca_parity_short_filament_variants_preserve_zero_slots_in_complete_output() {
    if std::env::var("ARES_PARITY_FILAMENT_VARIANTS_REFERENCE").as_deref() != Ok("1") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(&profiles_root(), "BBL").unwrap();
    let selection = parity::select_printer(&profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), Value::String("Manual".into()));
    machine.insert("filament_map".into(), serde_json::json!(["1", "1"]));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_prime_tower".into(), Value::String("0".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    for (filament, colour) in filaments.iter_mut().zip(["#F2754E", "#0000FF"]) {
        filament.insert("filament_colour".into(), serde_json::json!([colour]));
        filament.insert("filament_density".into(), serde_json::json!(["0"]));
        filament.insert("filament_cost".into(), serde_json::json!(["0"]));
        filament.insert(
            "filament_z_hop_types".into(),
            serde_json::json!(["Spiral Lift"]),
        );
        filament.insert("filament_wipe".into(), serde_json::json!(["1"]));
    }
    let label = "filament-variants/short-payload-zero-slots";
    let mut exported = runner
        .export_case(
            &CaseInputs {
                label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &runner::repo_root().join("tests/parity/cube10.stl"),
        )
        .unwrap();
    exported.project = shorten_payloads(&exported.project);
    let exported = runner
        .bind_filament_map_mode(exported, &Value::String("Manual".into()))
        .unwrap();
    runner::filament_map::verify(&exported.project, &Value::String("Manual".into())).unwrap();
    let settings = runner::application::settings(&exported.project).unwrap();
    assert_eq!(settings["filament_diameter"].as_array().unwrap().len(), 2);
    assert_eq!(settings["filament_map_mode"], "Manual");
    for key in [
        "filament_z_hop_types",
        "filament_wipe",
        "volumetric_speed_coefficients",
    ] {
        assert_eq!(settings[key].as_array().unwrap().len(), 2);
    }
    let sliced = runner.slice_case(&exported).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(native.contains("\n; activate_air_filtration_during_print = 1,0\n"));
    assert!(native.contains("\n; filament_ironing_flow = nil,0%\n"));
    assert!(native.contains("\n; filament_z_hop_types = Spiral Lift,Auto Lift\n"));
    assert!(native.contains("\n; filament_wipe = 1,0\n"));
    assert!(native.contains("\n; volumetric_speed_coefficients = \"0 0 0 0 0 0\";\n"));
    assert!(!native.contains("\n; filament used [g] = "));
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
