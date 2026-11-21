use super::super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root, volume_fixture};
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;
use std::io::{Cursor, Read};

#[path = "bridge_flow/fixture.rs"]
mod fixture;

const LANDED_SETTINGS: [(&str, &str); 12] = [
    ("enable_support", "1"),
    ("support_type", "normal(auto)"),
    ("support_style", "grid"),
    ("support_angle", "0"),
    ("support_on_build_plate_only", "0"),
    ("independent_support_layer_height", "1"),
    ("support_top_z_distance", "0.15"),
    ("support_bottom_z_distance", "0.2"),
    ("support_interface_top_layers", "3"),
    ("support_interface_bottom_layers", "2"),
    ("thick_bridges", "1"),
    ("bridge_flow", "1"),
];

fn export_landed_case(runner: &OrcaRunner) -> ExportedCase {
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    assert_eq!(machine["nozzle_diameter"], json!(["0.4"]));
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    for (key, value) in LANDED_SETTINGS {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let stl_path = runner.work.join("overhang.stl");
    std::fs::write(&stl_path, fixture::landed_overhang()).unwrap();
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "normal-support/modifier-bridge-flow",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &stl_path,
        )
        .unwrap();
    for (key, value) in LANDED_SETTINGS {
        application::verify_value(&exported.project, key, &json!(value)).unwrap();
    }
    let loaded_baseline = ares_core::load_project(&exported.project).unwrap();
    assert_eq!(loaded_baseline.settings().process.region.bridge_flow.0, 1.0);
    assert!(loaded_baseline.settings().process.object.thick_bridges.0);
    exported
}

#[test]
fn landed_support_preserves_complete_output() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = export_landed_case(&runner);
    let mut sliced = runner.slice_case(&exported).unwrap();
    sliced.label.push_str("/landed-control");
    let outcome = parity::compare_case(&sliced);
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(crate::option_coverage::normal_support_output::has_support_extrusion(native));
    assert!(interface_extrusion_at_height(native, 0.4));
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

#[test]
fn support_modifier_bridge_flow_averages_printing_regions() {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let exported = export_landed_case(&runner);
    let mut baseline = runner.slice_case(&exported).unwrap();
    baseline.label.push_str("/without-modifier");
    let baseline_outcome = parity::compare_case(&baseline);
    let project = volume_fixture::add_volumes(
        &exported.project,
        "modifier_part",
        &[(0.6, -6.0)],
        &[("bridge_flow", "2")],
    );
    for (key, value) in LANDED_SETTINGS {
        application::verify_value(&project, key, &json!(value)).unwrap();
    }
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].volumes().len(), 2);
    assert_eq!(
        loaded.objects()[0]
            .volumes()
            .iter()
            .map(|volume| (volume.id(), volume.volume_type()))
            .collect::<Vec<_>>(),
        [
            (1, ares_core::ProjectVolumeType::ModelPart),
            (3, ares_core::ProjectVolumeType::ParameterModifier),
        ]
    );
    assert_eq!(
        loaded.objects()[0].volumes()[1]
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
    let (_, modifier_xml) = xml
        .split_once("<part id=\"3\" subtype=\"modifier_part\">")
        .unwrap();
    let (modifier_xml, _) = modifier_xml.split_once("</part>").unwrap();
    let metadata = regex::Regex::new(r#"<metadata key="([^"]+)" value="([^"]*)"/>"#).unwrap();
    let mut values = metadata
        .captures_iter(modifier_xml)
        .map(|capture| (capture[1].to_owned(), capture[2].to_owned()))
        .collect::<Vec<_>>();
    values.sort();
    assert_eq!(
        values,
        [
            ("bridge_flow".into(), "2".into()),
            (
                "matrix".into(),
                "0.6 0 0 -6 0 1 0 0 0 0 1 7.5 0 0 0 1".into()
            ),
            ("name".into(), "overhang-annotation-3".into()),
        ]
    );
    let project_path = runner.work.join("bridge-flow-modifier.3mf");
    let output_dir = runner.work.join("bridge-flow-modifier-native");
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
    let control = std::str::from_utf8(&baseline.reference).unwrap();
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(interface_extrusion_at_height(control, 0.4));
    assert!(!interface_extrusion_at_height(control, 0.6));
    assert!(interface_extrusion_at_height(native, 0.6));
    assert_eq!(
        baseline_outcome.status, "PASS",
        "{}",
        baseline_outcome.detail
    );
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn interface_extrusion_at_height(reference: &str, expected: f64) -> bool {
    let mut interface = false;
    let mut height = 0.0;
    let mut relative_e = false;
    reference.lines().any(|line| {
        if let Some(role) = line.strip_prefix(";TYPE:") {
            interface = role == "Support interface";
        }
        if let Some(value) = line.strip_prefix(";HEIGHT:") {
            height = value.parse::<f64>().unwrap();
        }
        let instruction = line.split(';').next().unwrap();
        let words = instruction.split_ascii_whitespace();
        match words.clone().next() {
            Some("M83") => relative_e = true,
            Some("M82") => relative_e = false,
            _ => {}
        }
        interface
            && height == expected
            && relative_e
            && instruction.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.clone().any(|word| {
                word.strip_prefix('E')
                    .is_some_and(|value| value.parse::<f64>().unwrap() > 0.0)
            })
    })
}
