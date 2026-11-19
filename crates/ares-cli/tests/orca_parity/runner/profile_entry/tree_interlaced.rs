use super::{plater_archive::rewrite_archive, tree_extra_wall_areas::half_scale_instance};
use crate::{
    OrcaRunner,
    presets::VendorProfiles,
    runner::{CaseInputs, application::settings, native_api_cli::NativeCase, repo_root},
};
use ares_core::GenerationMetadata;
use serde_json::{Map, json};

#[test]
fn slim_interlaced_overhang_alternates_roof_angles() {
    compare_interfaces("overhang", "overhang.stl", "4.8");
}

#[test]
fn slim_interlaced_ledge_alternates_roof_angles() {
    compare_interfaces("ledge", "tree-ledge.stl", "4.8");
}

#[test]
fn slim_interlaced_shelf_starts_floor_angles_independently() {
    compare_interfaces("shelf", "tree-printed-shelf.stl", "2.675");
}

fn compare_interfaces(shape: &str, stl: &str, contact_z: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let repo = repo_root();
    let profiles =
        VendorProfiles::load(&repo.join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let export = runner
        .export_case(
            &CaseInputs {
                label: "x1c-slim-tree-interface",
                machine: &profiles.machine("Bambu Lab X1 Carbon 0.4 nozzle").unwrap(),
                process: &profiles.process("0.20mm Standard @BBL X1C").unwrap(),
                filaments: &[profiles.filament("Bambu PLA Basic @BBL X1C").unwrap()],
            },
            &Map::new(),
            &repo.join("tests/parity").join(stl),
        )
        .unwrap();
    let mut config = settings(&export.project).unwrap();
    config.insert("farthest_point_timelapse".into(), json!("0"));
    config.insert("enable_support".into(), json!("1"));
    config.insert("support_type".into(), json!("tree(auto)"));
    config.insert("support_style".into(), json!("tree_slim"));
    config.insert("support_on_build_plate_only".into(), json!("0"));
    if shape == "shelf" {
        config.insert("support_interface_bottom_layers".into(), json!("2"));
    }

    for pattern in [None, Some("rectilinear_interlaced")] {
        if let Some(pattern) = pattern {
            config.insert("support_interface_pattern".into(), json!(pattern));
        }
        let project = rewrite_archive(&export.project, |entry, text| match entry {
            "Metadata/project_settings.config" => Some(serde_json::to_string(&config).unwrap()),
            "3D/3dmodel.model" if shape == "overhang" => Some(half_scale_instance(text)),
            _ => None,
        });
        let label = format!("tree-interface-{shape}-{}-", pattern.unwrap_or("stock"));
        let case = NativeCase::slice(&runner, &project, &label);
        let (_, contact) = case
            .native
            .split_once(&format!("; Z_HEIGHT: {contact_z}\n"))
            .unwrap();
        assert!(
            contact
                .split("; Z_HEIGHT: ")
                .next()
                .unwrap()
                .contains("; FEATURE: Support interface\n")
        );
        case.compare(GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0));
    }
}
