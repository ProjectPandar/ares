use super::super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root, volume_fixture};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;
use std::io::{Cursor, Read};

#[test]
fn modifier_density_activates_inherited_second_filament() {
    if !crate::runner::reference_enabled("ARES_PARITY_REGION_FILAMENT_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Snapmaker",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Snapmaker", "Snapmaker J1 (0.4 nozzle)").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    assert_eq!(machine["nozzle_diameter"], json!(["0.4", "0.4"]));
    machine.insert("filament_map_mode".into(), json!("Manual"));
    machine.insert("filament_map".into(), json!(["1", "2"]));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "0"),
        ("enable_prime_tower", "0"),
        ("sparse_infill_density", "0%"),
        ("sparse_infill_pattern", "rectilinear"),
        ("sparse_infill_filament_id", "2"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = vec![profiles.filament(&selection.filaments[0]).unwrap(); 2];
    parity::normalize_filament_defaults(&mut filaments);
    for (filament, colour) in filaments.iter_mut().zip(["#F2754E", "#0000FF"]) {
        filament.insert("filament_colour".into(), json!([colour]));
    }
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "compatible-regions/inherited-second-filament",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    let exported = runner
        .bind_filament_map_mode(exported, &json!("Manual"))
        .unwrap();
    for (key, value) in settings {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    application::verify_value(
        &exported.project,
        "filament_colour",
        &json!(["#F2754E", "#0000FF"]),
    )
    .unwrap();
    application::verify_value(&exported.project, "filament_map", &json!(["1", "2"])).unwrap();
    let baseline = runner.slice_case(&exported).unwrap();
    let baseline_outcome = parity::compare_case(&baseline);
    assert!(!sparse_extrusion_on_second_tool(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));

    let project = volume_fixture::add_volumes(
        &exported.project,
        "modifier_part",
        &[(0.6, -6.0)],
        &[("sparse_infill_density", "50%")],
    );
    for (key, value) in settings {
        application::verify_value(&project, key, &json!(value)).unwrap();
    }
    super::super::filament_map::verify(&project, &json!("Manual")).unwrap();
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    let volumes = loaded.objects()[0].volumes();
    assert_eq!(volumes.len(), 2);
    assert_eq!((volumes[0].id(), volumes[1].id()), (1, 3));
    assert_eq!(
        volumes[0].volume_type(),
        ares_core::ProjectVolumeType::ModelPart
    );
    assert_eq!(
        volumes[1].volume_type(),
        ares_core::ProjectVolumeType::ParameterModifier
    );
    assert_eq!(
        volumes[1]
            .transform()
            .transform_point(ares_core::Point3d::new(1.0, 0.0, 0.0)),
        ares_core::Point3d::new(-5.4, 0.0, 7.5)
    );
    let mut archive = zip::ZipArchive::new(Cursor::new(&project)).unwrap();
    let mut xml = String::new();
    archive
        .by_name("Metadata/model_settings.config")
        .unwrap()
        .read_to_string(&mut xml)
        .unwrap();
    let (_, modifier) = xml
        .split_once("<part id=\"3\" subtype=\"modifier_part\">")
        .unwrap();
    let (modifier, _) = modifier.split_once("</part>").unwrap();
    let metadata = regex::Regex::new(r#"<metadata key="([^"]+)" value="([^"]*)"/>"#).unwrap();
    let mut keys = metadata
        .captures_iter(modifier)
        .map(|capture| capture[1].to_owned())
        .collect::<Vec<_>>();
    keys.sort();
    assert_eq!(keys, ["matrix", "name", "sparse_infill_density"]);
    assert!(modifier.contains("key=\"sparse_infill_density\" value=\"50%\""));
    let project_path = runner.work.join("inherited-filament.3mf");
    let output_dir = runner.work.join("inherited-filament-native");
    std::fs::write(&project_path, &project).unwrap();
    std::fs::create_dir_all(&output_dir).unwrap();
    let sliced = runner
        .slice_case(&ExportedCase {
            label: exported.label,
            project,
            project_path,
            output_dir,
        })
        .unwrap();
    let outcome = parity::compare_case(&sliced);
    assert!(sparse_extrusion_on_second_tool(
        std::str::from_utf8(&sliced.reference).unwrap()
    ));
    assert_eq!(
        (baseline_outcome.status, outcome.status),
        ("PASS", "PASS"),
        "baseline: {}; modifier: {}",
        baseline_outcome.detail,
        outcome.detail
    );
}

fn sparse_extrusion_on_second_tool(reference: &str) -> bool {
    let mut tool = None;
    let mut sparse = false;
    let mut relative_e = false;
    reference.lines().any(|line| {
        let instruction = line.split(';').next().unwrap().trim();
        let mut words = instruction.split_ascii_whitespace();
        match words.next() {
            Some("M83") => relative_e = true,
            Some("M82") => relative_e = false,
            Some("T0") => tool = Some(0),
            Some("T1") => tool = Some(1),
            _ => {}
        }
        if let Some(role) = line.strip_prefix(";TYPE:") {
            sparse = role == "Sparse infill";
        }
        tool == Some(1)
            && relative_e
            && sparse
            && instruction.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            })
    })
}
