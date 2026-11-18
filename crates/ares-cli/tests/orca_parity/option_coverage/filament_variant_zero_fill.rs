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
fn short_filament_variants_preserve_zero_slots_in_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(&profiles_root(), "BBL").unwrap();
    let selection = parity::select_printer(&profiles, "BBL", "Bambu Lab H2D 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    machine.insert("filament_map_mode".into(), Value::String("Manual".into()));
    machine.insert("filament_map".into(), serde_json::json!(["1", "1"]));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    process.insert("enable_prime_tower".into(), Value::String("0".into()));
    process.insert("disable_m73".into(), Value::String("1".into()));
    let mut filaments = vec![
        profiles
            .filament("Bambu PLA Basic @BBL H2D 0.2 nozzle")
            .unwrap();
        2
    ];
    // PrintConfig.cpp:11430–11456 expands logical-filament payloads through
    // filament_self_index before selecting the active variants.
    for filament in &mut filaments {
        filament.insert(
            "filament_extruder_variant".into(),
            serde_json::json!(["Direct Drive Standard", "Direct Drive High Flow"]),
        );
    }
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
    assert_eq!(
        settings["filament_diameter"].as_array().unwrap().len(),
        filaments.len()
    );
    assert_eq!(settings["filament_map_mode"], "Manual");
    for key in [
        "filament_z_hop_types",
        "filament_wipe",
        "volumetric_speed_coefficients",
    ] {
        assert_eq!(settings[key].as_array().unwrap().len(), filaments.len());
    }
    let sliced = runner.slice_case(&exported).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    // PrintConfig.cpp:11865–11882 copies an existing source value per slot,
    // including the first on a missing index. Uniform inputs stay uniform.
    for key in [
        "activate_air_filtration_during_print",
        "filament_z_hop_types",
        "filament_wipe",
        "volumetric_speed_coefficients",
    ] {
        let values = settings[key].as_array().unwrap();
        let source = &values[0];
        assert!(values.iter().all(|value| value == source));
        // Config.hpp:1207–1209 quotes the fixture's space-delimited strings.
        let (value, separator) = if key == "volumetric_speed_coefficients" {
            (serde_json::to_string(source).unwrap(), ";")
        } else {
            (source.as_str().unwrap().to_owned(), ",")
        };
        let expected = std::iter::repeat_n(value.as_str(), filaments.len())
            .collect::<Vec<_>>()
            .join(separator);
        assert!(native.contains(&format!("\n; {key} = {expected}\n")));
    }
    // GCode.cpp:8342–8351 omits options whose values are all nil.
    assert!(
        settings["filament_ironing_flow"]
            .as_array()
            .unwrap()
            .iter()
            .all(|value| value == "nil")
    );
    assert!(!native.contains("\n; filament_ironing_flow = "));
    assert!(!native.contains("\n; filament used [g] = "));
    let outcome = parity::compare_case(&sliced);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
