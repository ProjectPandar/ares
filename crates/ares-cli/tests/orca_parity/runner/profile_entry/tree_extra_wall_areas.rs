use super::plater_archive::{attribute, rewrite_archive};
use crate::{
    OrcaRunner,
    presets::VendorProfiles,
    runner::{CaseInputs, application::settings, native_api_cli::NativeCase, repo_root},
};
use ares_core::GenerationMetadata;
use serde_json::{Map, json};

#[test]
fn tree_slim_places_extra_walls_like_native() {
    compare_half_scale_overhang("slim", "tree_slim");
}

#[test]
fn tree_hybrid_places_extra_walls_like_native() {
    compare_half_scale_overhang("hybrid", "tree_hybrid");
}

#[test]
fn tree_auto_half_scale_overhang_matches_native() {
    compare_half_scale_overhang("auto", "default");
}

fn compare_half_scale_overhang(label: &str, style: &str) {
    let Some(runner) = OrcaRunner::from_env().unwrap() else {
        return;
    };
    let repo = repo_root();
    let profiles =
        VendorProfiles::load(&repo.join("OrcaSlicer/resources/profiles"), "BBL").unwrap();
    let export = runner
        .export_case(
            &CaseInputs {
                label: "x1c-tree-wall-overhang",
                machine: &profiles.machine("Bambu Lab X1 Carbon 0.4 nozzle").unwrap(),
                process: &profiles.process("0.20mm Standard @BBL X1C").unwrap(),
                filaments: &[profiles.filament("Bambu PLA Basic @BBL X1C").unwrap()],
            },
            &Map::new(),
            &repo.join("tests/parity/overhang.stl"),
        )
        .unwrap();
    let mut config = settings(&export.project).unwrap();
    config.insert("farthest_point_timelapse".into(), json!("0"));
    config.insert("enable_support".into(), json!("1"));
    config.insert("support_style".into(), json!(style));
    let project = rewrite_archive(&export.project, |entry, text| match entry {
        "Metadata/project_settings.config" => Some(serde_json::to_string(&config).unwrap()),
        "3D/3dmodel.model" => Some(half_scale_instance(text)),
        _ => None,
    });
    let case = NativeCase::slice(
        &runner,
        &project,
        &format!("tree-extra-wall-areas-{label}-"),
    );
    assert!(case.native.lines().any(|line| line == "; FEATURE: Support"));
    case.compare(GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0));
}

fn half_scale_instance(model: &str) -> String {
    let (resources, build) = model.split_once("<build").unwrap();
    assert_eq!(build.matches("<item ").count(), 1);
    let transform = attribute(build, "transform");
    let translation = transform.strip_prefix("1 0 0 0 1 0 0 0 1 ").unwrap();
    let build = build.replacen(
        &format!(r#"transform="{transform}""#),
        &format!(r#"transform="0.5 0 0 0 0.5 0 0 0 0.5 {translation}""#),
        1,
    );
    format!("{resources}<build{build}")
}
