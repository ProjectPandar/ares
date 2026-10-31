use super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root, volume_fixture};
use crate::option_coverage::normal_support_output::has_support_extrusion;
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

#[path = "support_modifier/bridge_flow.rs"]
mod bridge_flow;

#[test]
fn support_modifier_regions_preserve_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_SUPPORT_MODIFIER_REFERENCE") {
        return;
    }
    let runner = OrcaRunner::from_env().unwrap().unwrap();
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let machine = profiles.machine(&selection.printer).unwrap();
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "1"),
        ("support_type", "normal(auto)"),
        ("support_style", "grid"),
        ("support_angle", "0"),
        ("independent_support_layer_height", "1"),
        ("support_top_z_distance", "0.15"),
        ("support_bottom_z_distance", "0.2"),
        ("support_interface_top_layers", "3"),
        ("support_interface_bottom_layers", "-1"),
    ];
    for (key, value) in settings {
        process.insert(key.into(), json!(value));
    }
    let mut filaments = selection
        .filaments
        .iter()
        .map(|name| profiles.filament(name).unwrap())
        .collect::<Vec<_>>();
    parity::normalize_filament_defaults(&mut filaments);
    let exported = runner
        .export_case(
            &CaseInputs {
                label: "normal-support/modifier-regions",
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    let mut baseline = runner.slice_case(&exported).unwrap();
    baseline.label.push_str("/without-modifier");
    let outcome = parity::compare_case(&baseline);
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
    assert!(has_support_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));
    assert!(!has_modifier_width_extrusion(
        std::str::from_utf8(&baseline.reference).unwrap()
    ));

    let project = volume_fixture::add_volumes(
        &exported.project,
        "modifier_part",
        &[(0.6, -6.0)],
        &[("line_width", "0.6"), ("outer_wall_line_width", "0.6")],
    );
    for (key, value) in settings {
        application::verify_value(&project, key, &json!(value)).unwrap();
    }
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].volumes().len(), 2);
    let modifier = &loaded.objects()[0].volumes()[1];
    assert_eq!(
        modifier.volume_type(),
        ares_core::ProjectVolumeType::ParameterModifier
    );
    assert_eq!(
        modifier
            .transform()
            .transform_point(ares_core::Point3d::new(1.0, 0.0, 0.0)),
        ares_core::Point3d::new(-5.4, 0.0, 7.5)
    );
    let project_path = runner.work.join("support-modifier.3mf");
    let output_dir = runner.work.join("support-modifier-native");
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
    let native = std::str::from_utf8(&sliced.reference).unwrap();
    assert!(has_support_extrusion(native));
    assert!(has_modifier_width_extrusion(native));
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn has_modifier_width_extrusion(reference: &str) -> bool {
    let mut outer_wall = false;
    let mut modifier_width = false;
    let mut relative_e = false;
    reference.lines().any(|line| {
        if let Some(role) = line.strip_prefix(";TYPE:") {
            outer_wall = role == "Outer wall";
        }
        if let Some(width) = line.strip_prefix(";WIDTH:") {
            modifier_width = width == "0.6";
        }
        let instruction = line.split(';').next().unwrap();
        let words = instruction.split_ascii_whitespace();
        match words.clone().next() {
            Some("M83") => relative_e = true,
            Some("M82") => relative_e = false,
            _ => {}
        }
        outer_wall
            && modifier_width
            && relative_e
            && instruction.starts_with("G1 ")
            && words.clone().any(|word| word.starts_with(['X', 'Y']))
            && words.clone().any(|word| {
                word.strip_prefix('E')
                    .and_then(|value| value.parse::<f64>().ok())
                    .is_some_and(|value| value > 0.0)
            })
    })
}
