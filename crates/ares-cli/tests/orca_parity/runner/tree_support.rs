use super::{CaseInputs, OrcaRunner, application, repo_root};
use crate::option_coverage::normal_support_output::has_support_extrusion;
use crate::{self as parity, presets::VendorProfiles};
use serde_json::json;

#[path = "tree_support/manual.rs"]
mod manual;
#[path = "tree_support/retraction.rs"]
mod retraction;
#[path = "tree_support/roof_layers.rs"]
mod roof_layers;

#[test]
fn default_tree_auto_overhang_preserves_complete_output() {
    compare_overhang("tree-support/auto/default/overhang", "default", &[]);
}

#[test]
fn grid_tree_auto_normalizes_style_and_preserves_complete_output() {
    compare_overhang("tree-support/auto/grid/overhang", "grid", &[]);
}

#[test]
fn snug_tree_auto_normalizes_style_and_preserves_complete_output() {
    compare_overhang("tree-support/auto/snug/overhang", "snug", &[]);
}

#[test]
fn organic_zero_branch_angle_preserves_complete_output() {
    compare_overhang(
        "tree-support/auto/organic/zero-branch-angle",
        "default",
        &[("tree_support_branch_angle_organic", "0")],
    );
}

#[test]
fn slim_tree_auto_overhang_preserves_complete_output() {
    compare_overhang("tree-support/auto/slim/overhang", "tree_slim", &[]);
}

#[test]
fn strong_tree_auto_overhang_preserves_complete_output() {
    compare_overhang("tree-support/auto/strong/overhang", "tree_strong", &[]);
}

#[test]
fn hybrid_tree_auto_overhang_preserves_complete_output() {
    compare_overhang("tree-support/auto/hybrid/overhang", "tree_hybrid", &[]);
}

#[test]
fn strong_tree_lightning_overhang_preserves_complete_output() {
    compare_overhang(
        "tree-support/auto/strong/lightning/overhang",
        "tree_strong",
        &[("support_base_pattern", "lightning")],
    );
}

#[test]
fn normal_snug_overhang_preserves_complete_output() {
    compare_overhang(
        "normal-support/auto/snug/overhang",
        "snug",
        &[("support_type", "normal(auto)")],
    );
}

#[test]
fn normal_snug_sparse_interfaces_preserve_complete_output() {
    compare_overhang(
        "normal-support/auto/snug/sparse-interfaces",
        "snug",
        &[
            ("support_type", "normal(auto)"),
            ("support_interface_top_layers", "6"),
            ("support_interface_spacing", "1"),
            ("support_bottom_interface_spacing", "0.7"),
        ],
    );
}

#[test]
fn default_tree_interface_filament_keeps_one_base_interface_layer() {
    compare_overhang(
        "tree-support/auto/default/interface-filament-base-interface",
        "default",
        &[
            ("support_filament", "0"),
            ("support_interface_filament", "1"),
        ],
    );
}

#[test]
fn normal_snug_interface_filament_keeps_one_base_interface_layer() {
    compare_overhang(
        "normal-support/auto/snug/interface-filament-base-interface",
        "snug",
        &[
            ("support_type", "normal(auto)"),
            ("support_filament", "0"),
            ("support_interface_filament", "1"),
        ],
    );
}

fn compare_overhang(label: &str, style: &str, overrides: &[(&str, &str)]) {
    let Some(sliced) = overhang_case(label, style, overrides) else {
        return;
    };
    let outcome = parity::compare_case(&sliced);
    assert!(has_support_extrusion(
        std::str::from_utf8(&sliced.reference).unwrap()
    ));
    assert_eq!(outcome.status, "PASS", "{}", outcome.detail);
}

fn overhang_case(
    label: &str,
    style: &str,
    overrides: &[(&str, &str)],
) -> Option<super::ParityCase> {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_SUPPORT_REFERENCE") {
        return None;
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
        ("support_type", "tree(auto)"),
        ("support_style", style),
        ("raft_layers", "0"),
        ("support_threshold_angle", "30"),
        ("independent_support_layer_height", "1"),
        ("support_top_z_distance", "0.15"),
        ("support_bottom_z_distance", "0.2"),
        ("support_interface_top_layers", "3"),
        ("support_interface_bottom_layers", "-1"),
    ];
    let settings = settings.into_iter().chain(overrides.iter().copied());
    for (key, value) in settings.clone() {
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
                label,
                machine: &machine,
                process: &process,
                filaments: &filaments,
            },
            &parity::smoke_overrides(),
            &repo_root().join("tests/parity/overhang.stl"),
        )
        .unwrap();
    for (key, _) in settings {
        application::verify_value(&exported.project, key, &process[key]).unwrap();
    }
    let loaded = ares_core::load_project(&exported.project).unwrap();
    assert_eq!(loaded.objects().len(), 1);
    assert_eq!(loaded.objects()[0].volumes().len(), 1);
    assert_eq!(
        loaded.objects()[0].volumes()[0].volume_type(),
        ares_core::ProjectVolumeType::ModelPart
    );
    Some(runner.slice_case(&exported).unwrap())
}
