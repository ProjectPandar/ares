use super::{CaseInputs, ExportedCase, OrcaRunner, application, repo_root};
use crate::option_coverage::normal_support_output::has_support_extrusion;
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

#[test]
fn support_blocker_volume_preserves_complete_output() {
    verify_annotation(
        "normal(auto)",
        (
            "support_blocker",
            ares_core::ProjectVolumeType::SupportBlocker,
        ),
        false,
        &[(1.0, 0.0)],
        false,
    );
}

#[test]
fn manual_support_enforcer_volume_preserves_complete_output() {
    verify_annotation(
        "normal(manual)",
        (
            "support_enforcer",
            ares_core::ProjectVolumeType::SupportEnforcer,
        ),
        true,
        &[(1.0, 0.0)],
        false,
    );
}

#[test]
fn mirrored_support_blocker_preserves_complete_output() {
    verify_annotation(
        "normal(auto)",
        (
            "support_blocker",
            ares_core::ProjectVolumeType::SupportBlocker,
        ),
        false,
        &[(-1.0, 0.0)],
        false,
    );
}

#[test]
fn overlapping_support_blockers_preserve_complete_output() {
    verify_annotation(
        "normal(auto)",
        (
            "support_blocker",
            ares_core::ProjectVolumeType::SupportBlocker,
        ),
        false,
        &[(0.6, -6.0), (0.6, 6.0)],
        false,
    );
}

#[test]
fn annotation_custom_end_layer_tags_preserve_complete_output() {
    verify_annotation(
        "normal(manual)",
        (
            "support_enforcer",
            ares_core::ProjectVolumeType::SupportEnforcer,
        ),
        true,
        &[(1.0, 0.0)],
        true,
    );
}

fn verify_annotation(
    support_type: &str,
    (subtype, kind): (&str, ares_core::ProjectVolumeType),
    support: bool,
    transforms: &[(f64, f64)],
    custom_end_tags: bool,
) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let profiles = VendorProfiles::load(
        &repo_root().join("OrcaSlicer/resources/profiles"),
        "Creality",
    )
    .unwrap();
    let selection =
        parity::select_printer(&profiles, "Creality", "Creality Ender-3 0.4 nozzle").unwrap();
    let mut machine = profiles.machine(&selection.printer).unwrap();
    if custom_end_tags {
        let end = machine["machine_end_gcode"].as_str().unwrap();
        machine.insert(
            "machine_end_gcode".into(),
            json!(format!("{end}\n; note ;LAYER_CHANGE\n;LAYER_CHANGE\n")),
        );
    }
    let mut process = profiles.process(&selection.process).unwrap();
    parity::normalize_process_defaults(&machine, &mut process);
    let settings = [
        ("enable_support", "1"),
        ("support_type", support_type),
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
    let label = format!("normal-support/annotation/{subtype}/{transforms:?}");
    let exported = runner
        .export_case(
            &CaseInputs {
                label: &label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    let mut baseline = runner.slice_case(&exported).unwrap();
    baseline.label.push_str("/without-annotation");
    let baseline_outcome = parity::compare_case(&baseline);
    assert_eq!(
        baseline_outcome.status, "PASS",
        "{}",
        baseline_outcome.detail
    );
    assert_eq!(
        has_support_extrusion(std::str::from_utf8(&baseline.reference).unwrap()),
        !support
    );
    let project = super::volume_fixture::add_volumes(&exported.project, subtype, transforms, &[]);
    for (key, value) in settings {
        application::verify_value(&project, key, &json!(value)).unwrap();
    }
    let loaded = ares_core::load_project(&project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].volumes().len(), transforms.len() + 1);
    for (volume, &(scale_x, shift_x)) in loaded.objects()[0].volumes()[1..].iter().zip(transforms) {
        assert_eq!(volume.volume_type(), kind);
        assert_eq!(
            volume
                .transform()
                .transform_point(ares_core::Point3d::new(1.0, 0.0, 0.0)),
            ares_core::Point3d::new(scale_x + shift_x, 0.0, 7.5)
        );
    }
    let project_path = runner.work.join("support-blocker.3mf");
    let output_dir = runner.work.join("support-blocker-native");
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
    assert_eq!(
        has_support_extrusion(std::str::from_utf8(&sliced.reference).unwrap()),
        support
    );
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}
