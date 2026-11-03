//! Public 3MF replay of `GCode.cpp::needs_retraction` Tree base containment.

const ROOF_BASE_TALL_SKIRT: &[(&str, &str)] = &[
    ("skirt_height", "40"),
    ("support_top_z_distance", "0"),
    ("support_filament", "0"),
    ("support_interface_filament", "1"),
];

#[test]
fn single_cpu_slim_tree_concentric_hole_preserves_extrusion_direction() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    let case = super::overhang_case(
        "tree-support/slim/concentric-hole-direction",
        "tree_slim",
        ROOF_BASE_TALL_SKIRT,
    )
    .unwrap();
    let expected = "G1 X101.097 Y111.056 F9000\nG1 F4800\nG1 X99.234 Y110.273 E.08786\nG1 X100.778 Y106.707 E.16895\nG1 X101.097 Y106.986 E.01844\nG1 X101.097 Y111.056 E.17697\n";
    assert!(
        std::str::from_utf8(&case.reference)
            .unwrap()
            .contains(expected)
    );
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    assert!(
        std::str::from_utf8(&output).unwrap().contains(expected),
        "Concentric hole traversal must preserve source extrusion direction"
    );
}

#[test]
fn single_cpu_slim_tree_roof_slice_does_not_suppress_retraction() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    let case = super::overhang_case(
        "tree-support/slim/roof-slice-retraction",
        "tree_slim",
        ROOF_BASE_TALL_SKIRT,
    )
    .unwrap();
    let expected = "G1 X113.459 Y104.498 E.08498\nG1 E-2.8 F3600\n;WIPE_START\n";
    assert!(
        std::str::from_utf8(&case.reference)
            .unwrap()
            .contains(expected)
    );
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    assert!(
        std::str::from_utf8(&output).unwrap().contains(expected),
        "Tree roof planner slice must not exempt support travel from retraction"
    );
}

#[test]
fn single_cpu_slim_tree_roof_base_tall_skirt_preserves_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    super::compare_overhang(
        "tree-support/slim/roof-base-tall-skirt",
        "tree_slim",
        ROOF_BASE_TALL_SKIRT,
    );
}

#[test]
fn single_cpu_slim_tree_tall_skirt_preserves_complete_output() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    super::compare_overhang(
        "tree-support/slim/tall-skirt",
        "tree_slim",
        &[("skirt_height", "40")],
    );
}

#[test]
fn single_cpu_slim_tree_contained_travel_does_not_retract() {
    if !crate::runner::reference_enabled("ARES_PARITY_TREE_CONTAINMENT_REFERENCE") {
        return;
    }
    let case =
        super::overhang_case("tree-support/slim/contained-travel", "tree_slim", &[]).unwrap();
    let expected = "G1 X121.501 Y104.809 E.01758\nG1 X121.182 Y114.889 F9000\n";
    assert!(
        std::str::from_utf8(&case.reference)
            .unwrap()
            .contains(expected)
    );
    let output = tokio::runtime::Builder::new_current_thread()
        .build()
        .unwrap()
        .block_on(crate::runner::application::slice_project(
            &case.project,
            ares_core::GenerationMetadata::deterministic(2026, 8, 27, 0, 0, 0),
            ares_core::SlicerContext::new(0),
        ))
        .unwrap();
    assert!(
        std::str::from_utf8(&output).unwrap().contains(expected),
        "Tree travel contained in one base area must not retract or lift"
    );
}
